use phabricator_api::types::Phid;
use phabricator_api::Client as ApiClient;
use std::collections::HashMap;
use std::sync::{Arc, Mutex, Weak};
use url::Url;

mod search;

mod task;
pub use task::Task;

mod project;
pub use project::Project;

pub mod tasksbuilder;
use tasksbuilder::TasksBuilder;

pub mod projectsbuilder;
use projectsbuilder::ProjectsBuilder;

#[derive(Clone, Debug)]
pub struct Client {
    inner: Arc<Inner>,
}

#[derive(Clone, Debug)]
pub(crate) struct WeakClient {
    inner: Weak<Inner>,
}

#[derive(Debug)]
pub(crate) struct Cache {
    tasks: HashMap<u32, Task>,
    projects: HashMap<Phid, Project>,
}

#[derive(Debug)]
struct Inner {
    client: ApiClient,
    // Changable data
    cache: Mutex<Cache>,
}

impl Client {
    pub fn new(base: Url, token: String) -> Self {
        let client = ApiClient::new(base, token);
        let tasks = HashMap::new();
        let projects = HashMap::new();
        let cache = Mutex::new(Cache { tasks, projects });
        let inner = Arc::new(Inner { client, cache });

        Self { inner }
    }

    pub(crate) fn downgrade(&self) -> WeakClient {
        WeakClient {
            inner: Arc::downgrade(&self.inner),
        }
    }

    pub fn tasks<'a, T>(&self, tasks: T) -> TasksBuilder<'_>
    where
        T: IntoIterator<Item = &'a u32> + 'a,
    {
        let mut iter = tasks.into_iter();
        TasksBuilder::new(self, &mut iter)
    }

    pub fn tasks_by_phid<'a, T>(&'a self, phids: &'a mut T) -> TasksBuilder<'a>
    where
        T: Iterator<Item = &'a Phid>,
    {
        TasksBuilder::new_by_phids(self, phids)
    }

    pub fn cached_task(&self, id: u32) -> Option<Task> {
        self.access_cache(|cache| cache.tasks.get(&id).cloned())
    }

    pub fn cached_task_by_phid(&self, phid: &Phid) -> Option<Task> {
        self.access_cache(|cache| cache.tasks.values().find(|t| t.phid() == phid).cloned())
    }

    pub fn cached_project(&self, phid: &Phid) -> Option<Project> {
        self.access_cache(|cache| cache.projects.get(phid).cloned())
    }

    pub(crate) fn client(&self) -> &ApiClient {
        &self.inner.client
    }

    pub fn projects_by_phid<'a, P>(&self, phids: P) -> ProjectsBuilder<'_, P>
    where
        P: IntoIterator<Item = &'a Phid>,
    {
        ProjectsBuilder::new(self, phids)
    }

    pub(crate) fn access_cache<T, F>(&self, f: F) -> T
    where
        F: FnOnce(&Cache) -> T,
    {
        let cache = self.inner.cache.lock().unwrap();
        f(&cache)
    }

    pub(crate) fn update_cache<T, F>(&self, f: F) -> T
    where
        F: FnOnce(&mut Cache) -> T,
    {
        let mut cache = self.inner.cache.lock().unwrap();
        f(&mut cache)
    }
}

impl WeakClient {
    pub(crate) fn upgrade(&self) -> Option<Client> {
        self.inner.upgrade().map(|inner| Client { inner })
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use futures::prelude::*;
    use phabricator_mock::task;
    use phabricator_mock::PhabMockServer;
    use rust_decimal::prelude::*;

    async fn setup() -> PhabMockServer {
        let m = PhabMockServer::start().await;
        let user = m.new_user("user", "Test User");

        let p = phabricator_mock::project()
            .id(10)
            .name("Project")
            .build()
            .unwrap();
        m.add_project(p.clone());

        let t100 = phabricator_mock::task()
            .id(100)
            .full_name("Test task 100")
            .description("100 test description")
            .points(Decimal::new(25, 2))
            .author(user.clone())
            .owner(user.clone())
            .status(m.default_status())
            .priority(m.default_priority())
            .projects(vec![p])
            .build()
            .unwrap();
        m.add_task(t100.clone());
        let t200 = m.new_simple_task(200, &user);

        task::link(&t100, &t200);

        let t300 = m.new_simple_task(300, &user);
        task::link(&t200, &t300);

        let t400 = m.new_simple_task(400, &user);
        task::link(&t200, &t400);

        m
    }

    #[tokio::test]
    async fn simple() {
        let m = setup().await;
        let client = Client::new(m.uri(), m.token().to_string());

        let mut result = client.tasks(&mut [100, 200].iter()).query();
        let mut seen = Vec::new();

        while let Some(t) = result.try_next().await.unwrap() {
            seen.push(t.id());
            let mock = m.get_task(t.id()).unwrap();
            assert_eq!(mock.full_name, t.title());
            assert_eq!(mock.description, t.description());
            assert_eq!(mock.points, t.points());
        }

        seen.sort();
        assert_eq!([100, 200], seen.as_slice());
        assert_eq!(1, m.n_requests().await);
    }

    #[tokio::test]
    async fn task_cache() {
        let m = setup().await;
        let client = Client::new(m.uri(), m.token().to_string());

        let result = client.tasks(&mut [100].iter()).query();

        let mut tasks: Vec<_> = result.try_collect().await.unwrap();
        assert_eq!(1, tasks.len());

        let task = tasks.pop().unwrap();
        assert_eq!(100, task.id());

        assert_eq!(1, m.n_requests().await);

        let result = client.tasks(&mut [100].iter()).query();
        let mut tasks: Vec<_> = result.try_collect().await.unwrap();
        assert_eq!(1, tasks.len());

        let task2 = tasks.pop().unwrap();
        assert_eq!(task.id(), task2.id());

        // No request to have been done
        assert_eq!(1, m.n_requests().await);
    }

    #[tokio::test]
    async fn projects_on_demand() {
        let m = setup().await;
        let client = Client::new(m.uri(), m.token().to_string());

        let mut tasks: Vec<Task> = client
            .tasks(&mut [100].iter())
            .query()
            .try_collect()
            .await
            .unwrap();

        assert_eq!(1, tasks.len());
        let task = tasks.pop().unwrap();
        assert_eq!(100, task.id());

        assert_eq!(1, m.n_requests().await);
        let mut projects = task.projects().await.unwrap();

        // Extra requests to resolve projects
        assert_eq!(3, m.n_requests().await);
        assert_eq!(1, projects.len());

        let project = projects.pop().unwrap();
        assert_eq!(10, project.id());
    }

    #[tokio::test]
    async fn projects_batch() {
        let m = setup().await;
        let client = Client::new(m.uri(), m.token().to_string());

        let mut tasks: Vec<Task> = client
            .tasks(&mut [100].iter())
            .projects()
            .query()
            .try_collect()
            .await
            .unwrap();

        assert_eq!(1, tasks.len());
        let task = tasks.pop().unwrap();
        assert_eq!(100, task.id());

        // Request for tasks and for projects
        assert_eq!(2, m.n_requests().await);
        let mut projects = task.projects().await.unwrap();

        // No requests needed to resolve projects
        assert_eq!(2, m.n_requests().await);
        assert_eq!(1, projects.len());

        let project = projects.pop().unwrap();
        assert_eq!(10, project.id());
    }

    #[tokio::test]
    async fn parent_tasks() {
        let m = setup().await;
        let client = Client::new(m.uri(), m.token().to_string());

        let mut tasks: Vec<Task> = client.tasks(&[200]).query().try_collect().await.unwrap();

        let task = tasks.pop().unwrap();
        assert_eq!(200, task.id());

        let mut parents = task.parents().await.unwrap();
        assert_eq!(1, parents.len());
        let parent = parents.pop().unwrap();
        assert_eq!(100, parent.id());
    }

    #[tokio::test]
    async fn sub_tasks() {
        let m = setup().await;
        let client = Client::new(m.uri(), m.token().to_string());

        let mut tasks: Vec<Task> = client.tasks(&[200]).query().try_collect().await.unwrap();

        let task = tasks.pop().unwrap();
        assert_eq!(200, task.id());

        let subtasks = task.subtasks().await.unwrap();
        assert_eq!(2, subtasks.len());
        let mut ids: Vec<_> = subtasks.iter().map(|t| t.id()).collect();
        ids.sort();
        assert_eq!(&[300, 400], ids.as_slice());
    }
}
