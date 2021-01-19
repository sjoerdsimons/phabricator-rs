use crate::Project;
use crate::{Client, WeakClient};
use futures::prelude::*;
use phabricator_api::edge::search::Search as EdgeSearch;
use phabricator_api::edge::search::Type as EdgeType;
use phabricator_api::maniphest::search::Projects;
use phabricator_api::maniphest::search::SearchData;
use phabricator_api::types::Phid;
use phabricator_api::RequestError;
use rust_decimal::prelude::*;
use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::Mutex;

#[derive(Clone, Debug)]
pub struct Task {
    id: u32,
    phid: Phid,
    client: WeakClient,
    inner: Arc<Mutex<Inner>>,
}

#[derive(Clone, Debug, Default)]
struct Inner {
    title: String,
    description: String,
    status: String,
    projects: Option<Vec<Project>>,
    parents: Option<Vec<Task>>,
    subtasks: Option<Vec<Task>>,
    points: Option<Decimal>,
}

impl Task {
    fn map_projects(projects: Projects, cache: &HashMap<Phid, Project>) -> Vec<Project> {
        projects
            .projects
            .iter()
            .map(|phid| cache[phid].clone())
            .collect()
    }

    fn from_searchdata(data: SearchData, client: &Client, cache: &HashMap<Phid, Project>) -> Task {
        let projects = data
            .attachments
            .projects
            .map(|p| Self::map_projects(p, cache));

        let inner = Arc::new(Mutex::new(Inner {
            title: data.fields.name,
            description: data.fields.description,
            points: data.fields.points,
            status: data.fields.status.value,
            projects,
            subtasks: None,
            parents: None,
        }));

        Task {
            id: data.id,
            client: client.downgrade(),
            phid: data.phid,
            inner,
        }
    }

    fn update_searchdata(&mut self, data: SearchData, cache: &HashMap<Phid, Project>) {
        let projects = data
            .attachments
            .projects
            .map(|p| Self::map_projects(p, cache));
        let mut inner = self.inner.lock().unwrap();
        inner.title = data.fields.name;
        inner.description = data.fields.description;
        inner.points = data.fields.points;
        inner.status = data.fields.status.value;
        inner.projects = projects;
        // TODO update other fields
    }

    pub(crate) fn update_from_searchdata(data: SearchData, client: &Client) -> Task {
        client.update_cache(|cache| match cache.tasks.entry(data.id) {
            Entry::Vacant(v) => v
                .insert(Self::from_searchdata(data, client, &cache.projects))
                .clone(),
            Entry::Occupied(mut o) => {
                let t = o.get_mut();
                t.update_searchdata(data, &cache.projects);
                t.clone()
            }
        })
    }

    pub fn id(&self) -> u32 {
        self.id
    }

    pub fn phid(&self) -> &Phid {
        &self.phid
    }

    pub fn title(&self) -> String {
        let l = self.inner.lock().unwrap();
        l.title.clone()
    }

    pub fn description(&self) -> String {
        let l = self.inner.lock().unwrap();
        l.description.clone()
    }

    pub fn status(&self) -> String {
        let l = self.inner.lock().unwrap();
        l.status.clone()
    }

    pub fn points(&self) -> Option<Decimal> {
        let l = self.inner.lock().unwrap();
        l.points
    }

    pub(crate) fn projects_resolved(&self) -> bool {
        let l = self.inner.lock().unwrap();
        l.projects.is_some()
    }

    pub async fn projects(&self) -> Result<Vec<Project>, RequestError> {
        {
            let l = self.inner.lock().unwrap();
            if let Some(ref projects) = l.projects {
                return Ok(projects.clone());
            }
        }
        let client = self.client.upgrade().unwrap();
        client
            .tasks(&[self.id])
            .projects()
            .query()
            .try_for_each(|_| future::ready(Ok(())))
            .await?;
        let l = self.inner.lock().unwrap();
        Ok(l.projects.as_ref().unwrap().clone())
    }

    async fn edges(&self, edge: EdgeType) -> Result<Vec<Task>, RequestError> {
        // TODO error handle
        let client = self.client.upgrade().unwrap();
        // TODO handle pagination
        let s = EdgeSearch {
            sources: vec![self.phid.clone()],
            types: vec![edge],
            ..Default::default()
        };

        // TODO error handle
        let r = client.client().request(&s).await?;

        let mut phids = r.data.iter().map(|data| &data.dest);

        let tasks = client
            .tasks_by_phid(&mut phids)
            .query()
            .try_collect()
            .await?;
        Ok(tasks)
    }

    pub async fn parents(&self) -> Result<Vec<Task>, RequestError> {
        {
            let l = self.inner.lock().unwrap();
            if let Some(ref parents) = l.parents {
                return Ok(parents.clone());
            }
        }
        let tasks = self.edges(EdgeType::TaskParent).await?;
        let mut l = self.inner.lock().unwrap();
        l.parents = Some(tasks);
        Ok(l.parents.as_ref().unwrap().clone())
    }

    pub async fn subtasks(&self) -> Result<Vec<Task>, RequestError> {
        {
            let l = self.inner.lock().unwrap();
            if let Some(ref subtasks) = l.subtasks {
                return Ok(subtasks.clone());
            }
        }
        let tasks = self.edges(EdgeType::TaskSubtask).await?;
        let mut l = self.inner.lock().unwrap();
        l.subtasks = Some(tasks);
        Ok(l.subtasks.as_ref().unwrap().clone())
    }
}
