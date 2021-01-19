use crate::search;
use crate::Client;
use crate::Project;
use futures::prelude::*;
use futures::Stream;
use phabricator_api::project::search::Search;
use phabricator_api::project::search::SearchCursor;
use phabricator_api::types::Cursor;
use phabricator_api::types::Phid;
use phabricator_api::RequestError;
use std::sync::Arc;

async fn get(
    client: &Client,
    search: Arc<Search>,
    cursor: Option<Cursor>,
) -> Result<search::QueryData<Project>, RequestError> {
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

    let projects = data
        .data
        .drain(..)
        .map(|d| Project::update_from_searchdata(d, client))
        .collect();

    let cursor = if data.cursor.after.is_some() {
        Some(data.cursor)
    } else {
        None
    };

    Ok((projects, cursor))
}

pub struct ProjectsBuilder<'c, P> {
    client: &'c Client,
    phids: P,
}

impl<'a, 'c, P> ProjectsBuilder<'c, P>
where
    P: IntoIterator<Item = &'a Phid>,
{
    pub(crate) fn new(client: &'c Client, phids: P) -> Self {
        ProjectsBuilder { client, phids }
    }

    pub fn query(self) -> impl Stream<Item = Result<Project, RequestError>> + 'c {
        let client = &self.client;
        let (lookup, cached) =
            self.phids
                .into_iter()
                .fold((vec![], vec![]), |(mut lookup, mut cached), p| {
                    let project = client.cached_project(p);

                    match project {
                        Some(prj) => cached.push(prj),
                        None => lookup.push(p.clone()),
                    }
                    (lookup, cached)
                });

        let search = if lookup.is_empty() {
            None
        } else {
            let mut search: Search = Default::default();
            search.constraints.phids = Some(lookup);
            Some(search)
        };

        search::Search::new(
            self.client,
            cached,
            search,
            Box::new(|client, search, cursor| get(client, search, cursor).boxed()),
        )
    }
}
