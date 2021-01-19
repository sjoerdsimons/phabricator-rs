use crate::{Client, WeakClient};
use phabricator_api::project::search::SearchData;
use phabricator_api::types::Phid;
use std::collections::hash_map::Entry;
use std::sync::Arc;
use std::sync::Mutex;

#[derive(Clone, Debug)]
pub struct Project {
    id: u32,
    phid: Phid,
    client: WeakClient,
    inner: Arc<Mutex<Inner>>,
}

#[derive(Clone, Debug)]
struct Inner {
    title: String,
    slug: Option<String>,
    description: Option<String>,
}

impl Project {
    fn from_searchdata(data: SearchData, client: &Client) -> Project {
        let inner = Arc::new(Mutex::new(Inner {
            title: data.fields.name,
            slug: data.fields.slug,
            description: data.fields.description,
        }));

        Project {
            id: data.id,
            phid: data.phid,
            client: client.downgrade(),
            inner,
        }
    }

    fn update_searchdata(&mut self, _data: SearchData) {
        todo!()
    }

    pub(crate) fn update_from_searchdata(data: SearchData, client: &Client) -> Project {
        client.update_cache(|cache| match cache.projects.entry(data.phid.clone()) {
            Entry::Vacant(v) => v.insert(Self::from_searchdata(data, client)).clone(),
            Entry::Occupied(mut o) => {
                let t = o.get_mut();
                t.update_searchdata(data);
                t.clone()
            }
        })
    }

    pub fn id(&self) -> u32 {
        self.id
    }

    pub fn title(&self) -> String {
        let l = self.inner.lock().unwrap();
        l.title.clone()
    }

    pub fn slug(&self) -> Option<String> {
        let l = self.inner.lock().unwrap();
        l.slug.clone()
    }

    pub fn description(&self) -> Option<String> {
        let l = self.inner.lock().unwrap();
        l.description.clone()
    }
}
