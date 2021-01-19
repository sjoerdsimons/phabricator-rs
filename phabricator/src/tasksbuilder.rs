use crate::search;
use crate::Client;
use crate::Task;
use futures::prelude::*;
use phabricator_api::maniphest::search::Search;
use phabricator_api::maniphest::search::SearchCursor;
use phabricator_api::types::Cursor;
use phabricator_api::types::Phid;
use phabricator_api::RequestError;
use std::collections::HashSet;
use std::sync::Arc;

async fn get(
    client: &Client,
    search: Arc<Search>,
    cursor: Option<Cursor>,
) -> Result<search::QueryData<Task>, RequestError> {
    let mut data = match cursor {
        Some(cursor) => {
            let s = SearchCursor {
                cursor: &cursor,
                search: &*search,
            };
            client.client().request(&s).await?
        }
        None => client.client().request(&*search).await?,
    };
    let projects: HashSet<_> = data
        .data
        .iter()
        .filter_map(|d| d.attachments.projects.as_ref().map(|p| &p.projects))
        .flatten()
        .collect();

    if !projects.is_empty() {
        client
            .projects_by_phid(projects)
            .query()
            .try_for_each(|_| future::ready(Ok(())))
            .await?;
    }

    let tasks = data
        .data
        .drain(..)
        .map(|d| Task::update_from_searchdata(d, client))
        .collect();

    let cursor = if data.cursor.after.is_some() {
        Some(data.cursor)
    } else {
        None
    };

    Ok((tasks, cursor))
}

enum Constraint<'a> {
    Tasks(Vec<u32>),
    Phids(Vec<&'a Phid>),
}

pub struct TasksBuilder<'c> {
    client: &'c Client,
    constraints: Constraint<'c>,
    resolve_projects: bool,
}

impl<'a, 'c> TasksBuilder<'c> {
    pub(crate) fn new(client: &'c Client, tasks: &mut dyn Iterator<Item = &u32>) -> Self {
        TasksBuilder {
            client,
            constraints: Constraint::Tasks(tasks.copied().collect()),
            resolve_projects: false,
        }
    }

    pub(crate) fn new_by_phids(
        client: &'c Client,
        phids: &'c mut dyn Iterator<Item = &'c Phid>,
    ) -> Self {
        TasksBuilder {
            client,
            constraints: Constraint::Phids(phids.collect()),
            resolve_projects: false,
        }
    }

    pub fn projects(mut self) -> Self {
        self.resolve_projects = true;
        self
    }

    pub fn query(self) -> impl Stream<Item = Result<Task, RequestError>> + 'c {
        let client = &self.client;
        let resolve_projects = self.resolve_projects;

        let mut constraints = self.constraints;
        let (search, cached) = match constraints {
            Constraint::Tasks(ref mut tasks) => {
                let (lookup, cached) =
                    tasks
                        .iter()
                        .fold((vec![], vec![]), |(mut lookup, mut cached), t| {
                            let task = client.cached_task(*t).filter(|t| {
                                if resolve_projects {
                                    t.projects_resolved()
                                } else {
                                    true
                                }
                            });

                            match task {
                                Some(t) => cached.push(t),
                                None => lookup.push(*t),
                            }
                            (lookup, cached)
                        });
                let search = if lookup.is_empty() {
                    None
                } else {
                    let mut search: Search = Default::default();
                    search.constraints.ids = Some(lookup);
                    search.attachments.projects = self.resolve_projects;
                    Some(search)
                };
                (search, cached)
            }
            Constraint::Phids(ref mut phids) => {
                let (lookup, cached) =
                    phids
                        .iter()
                        .fold((vec![], vec![]), |(mut lookup, mut cached), p| {
                            let task = client.cached_task_by_phid(p).filter(|t| {
                                if resolve_projects {
                                    t.projects_resolved()
                                } else {
                                    true
                                }
                            });

                            match task {
                                Some(t) => cached.push(t),
                                None => lookup.push((*p).clone()),
                            }
                            (lookup, cached)
                        });
                let search = if lookup.is_empty() {
                    None
                } else {
                    let mut search: Search = Default::default();
                    search.constraints.phids = Some(lookup);
                    search.attachments.projects = self.resolve_projects;
                    Some(search)
                };
                (search, cached)
            }
        };

        search::Search::new(
            self.client,
            cached,
            search,
            Box::new(|client, search, cursor| get(client, search, cursor).boxed()),
        )
    }
}
