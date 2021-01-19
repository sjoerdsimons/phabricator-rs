use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use url::Url;
use wiremock::matchers::{header, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};
use wiremock::{Request, Respond};

mod param;
use param::Params;

pub mod task;
use task::Task;

mod status;
use status::Status;

mod user;
use user::User;

mod priority;
use priority::Priority;

mod policy;
use policy::Policy;

mod space;
use space::Space;

pub mod project;
use project::Project;

mod api;
mod column;
use column::Column;

mod phid;
use phid::Phid;

pub fn status() -> status::StatusBuilder {
    status::StatusBuilder::default()
}

pub fn column() -> column::ColumnDataBuilder {
    column::ColumnDataBuilder::default()
}

pub fn project() -> project::ProjectDataBuilder {
    project::ProjectDataBuilder::default()
}

pub fn priority() -> priority::PriorityBuilder {
    priority::PriorityBuilder::default()
}

pub fn task() -> task::TaskDataBuilder {
    task::TaskDataBuilder::default()
}

pub fn user() -> user::UserDataBuilder {
    user::UserDataBuilder::default()
}

trait PhabRespond: Send + Sync {
    fn respond(
        &self,
        server: &PhabMockServer,
        params: &Params,
        request: &Request,
    ) -> ResponseTemplate;
}

struct AuthAndParse<R> {
    server: PhabMockServer,
    responder: R,
}

impl<R> Respond for AuthAndParse<R>
where
    R: PhabRespond,
{
    fn respond(&self, request: &Request) -> ResponseTemplate {
        let params = Params::new(&request.body).expect("Failed to parse request");
        let auth = params.get(&["api.token"]);

        match auth {
            None => ResponseTemplate::new(403).set_body_string("Missing auth token"),
            Some(a) if a != self.server.token() => {
                ResponseTemplate::new(403).set_body_string("Incorrect auth token")
            }
            _ => self.responder.respond(&self.server, &params, request),
        }
    }
}

struct Data {
    tasks: HashMap<u32, Task>,
    users: Vec<User>,
    default_priority: Priority,
    priorities: Vec<Priority>,
    statusses: Vec<Status>,
    projects: Vec<Project>,
}

struct Inner {
    server: MockServer,
    token: String,
    data: Mutex<Data>,
}

#[derive(Clone)]
pub struct PhabMockServer {
    inner: Arc<Inner>,
}

impl PhabMockServer {
    fn auth_and_parse<R>(&self, responder: R) -> AuthAndParse<R>
    where
        R: PhabRespond,
    {
        AuthAndParse {
            server: self.clone(),
            responder,
        }
    }

    async fn handle_post<R>(&self, p: &str, responder: R)
    where
        R: PhabRespond + 'static,
    {
        Mock::given(method("POST"))
            .and(path(p))
            .and(header("content-type", "application/x-www-form-urlencoded"))
            .respond_with(self.auth_and_parse(responder))
            .named("phid.lookup")
            .mount(&self.inner.server)
            .await;
    }

    pub async fn start() -> Self {
        let server = MockServer::start().await;

        let default_priority = Priority {
            value: 50,
            name: "normal".to_string(),
            color: "yellow".to_string(),
        };

        let data = Data {
            tasks: HashMap::new(),
            users: Vec::new(),
            default_priority,
            priorities: Vec::new(),
            statusses: Vec::new(),
            projects: Vec::new(),
        };
        let m = PhabMockServer {
            inner: Arc::new(Inner {
                server,
                token: "badgerbadger".to_string(),
                data: Mutex::new(data),
            }),
        };

        m.new_priority(10, "Low", "blue");
        m.new_priority(100, "High", "blue");

        let s = status()
            .value("open")
            .name("Open")
            .color("green")
            .special(status::Special::Default)
            .build()
            .unwrap();
        m.add_status(s);
        m.new_status("wip", "In Progress", "indigo");
        let s = status()
            .value("closed")
            .name("Closed")
            .color("indigo")
            .special(status::Special::Closed)
            .closed(true)
            .build()
            .unwrap();
        m.add_status(s);

        m.handle_post("api/maniphest.search", api::maniphest::Search {})
            .await;
        m.handle_post("api/maniphest.info", api::maniphest::Info {})
            .await;
        m.handle_post("api/phid.lookup", api::phid::Lookup {}).await;
        m.handle_post("api/project.search", api::project::Search {})
            .await;
        m.handle_post("api/edge.search", api::edge::Search {}).await;
        m
    }

    pub fn uri(&self) -> Url {
        self.inner.server.uri().parse().expect("uri not a url")
    }

    pub fn token(&self) -> &str {
        &self.inner.token
    }

    pub async fn n_requests(&self) -> usize {
        self.inner
            .server
            .received_requests()
            .await
            .map(|v| v.len())
            .unwrap_or_default()
    }

    pub async fn requests(&self) -> Option<Vec<wiremock::Request>> {
        self.inner.server.received_requests().await
    }

    pub fn add_task(&self, task: Task) {
        let mut data = self.inner.data.lock().unwrap();
        data.tasks.insert(task.id, task);
    }

    pub fn add_project(&self, project: Project) {
        let mut data = self.inner.data.lock().unwrap();
        data.projects.push(project);
    }

    pub fn add_user(&self, u: User) {
        let mut data = self.inner.data.lock().unwrap();
        data.users.push(u);
    }

    pub fn add_status(&self, s: Status) {
        let mut data = self.inner.data.lock().unwrap();
        data.statusses.push(s);
    }

    pub fn add_priority(&self, p: Priority) {
        let mut data = self.inner.data.lock().unwrap();
        data.priorities.push(p);
    }

    pub fn get_task(&self, id: u32) -> Option<Task> {
        let data = self.inner.data.lock().unwrap();
        match data.tasks.get(&id) {
            Some(t) => Some(t.clone()),
            None => None,
        }
    }

    pub fn find_task(&self, phid: &Phid) -> Option<Task> {
        let data = self.inner.data.lock().unwrap();
        data.tasks
            .values()
            .find(|t| t.phid == *phid)
            .map(Clone::clone)
    }

    pub fn get_project(&self, id: u32) -> Option<Project> {
        let data = self.inner.data.lock().unwrap();
        data.projects.iter().find(|p| p.id == id).map(Clone::clone)
    }

    pub fn find_project(&self, phid: &Phid) -> Option<Project> {
        let data = self.inner.data.lock().unwrap();
        data.projects
            .iter()
            .find(|p| p.phid == *phid)
            .map(Clone::clone)
    }

    pub fn default_status(&self) -> Status {
        let data = self.inner.data.lock().unwrap();
        data.statusses
            .iter()
            .find(|s| s.special == Some(status::Special::Default))
            .map(Clone::clone)
            .expect("No default status")
    }

    pub fn default_priority(&self) -> Priority {
        let data = self.inner.data.lock().unwrap();
        data.default_priority.clone()
    }

    pub fn new_user(&self, name: &str, full_name: &str) -> User {
        let u = user::UserDataBuilder::default()
            .full_name(full_name)
            .name(name)
            .build()
            .unwrap();
        self.add_user(u.clone());
        u
    }

    pub fn new_priority(&self, value: u32, name: &str, color: &str) -> Priority {
        let p = priority()
            .value(value)
            .name(name)
            .color(color)
            .build()
            .unwrap();
        self.add_priority(p.clone());
        p
    }

    pub fn new_status(&self, value: &str, name: &str, color: &str) -> Status {
        let s = status()
            .value(value)
            .name(name)
            .color(color)
            .build()
            .unwrap();
        self.add_status(s.clone());
        s
    }

    pub fn new_simple_task(&self, id: u32, user: &User) -> Task {
        let task = task()
            .id(id)
            .full_name(format!("Task T{}", id))
            .description(format!("Description of task T{}", id))
            .author(user.clone())
            .owner(user.clone())
            .priority(self.default_priority())
            .status(self.default_status())
            .build()
            .unwrap();
        self.add_task(task.clone());
        task
    }
}
