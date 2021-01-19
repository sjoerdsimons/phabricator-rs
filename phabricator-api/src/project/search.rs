use crate::types::Phid;
use crate::utils::deserialize_timestamp;
use crate::ApiRequest;
use chrono::DateTime;
use chrono::Utc;
use serde::Deserialize;
use serde::Serialize;
use serde_json::value::Value as JsonValue;
use std::collections::HashMap;
use std::default::Default;
use std::ops::Not;

pub trait Serializable: erased_serde::Serialize + std::fmt::Debug {}
impl<T: erased_serde::Serialize + std::fmt::Debug> Serializable for T {}
erased_serde::serialize_trait_object!(Serializable);

#[derive(Serialize, Debug, Default)]
pub struct Constraints {
    pub ids: Option<Vec<u32>>,
    pub slugs: Option<Vec<String>>,
    pub query: Option<String>,
    pub phids: Option<Vec<Phid>>,
    #[serde(flatten)]
    pub custom: Option<HashMap<String, Box<dyn Serializable + Send + Sync>>>,
}

#[derive(Serialize, Debug, Default)]
pub struct Attachments {
    #[serde(skip_serializing_if = "<&bool>::not")]
    pub members: bool,
    #[serde(skip_serializing_if = "<&bool>::not")]
    pub watchers: bool,
    #[serde(skip_serializing_if = "<&bool>::not")]
    pub ancestors: bool,
}

pub type Search = crate::types::Search<Attachments, Constraints>;
pub type SearchCursor<'a> = crate::types::SearchCursor<'a, Attachments, Constraints>;

pub type SearchData = crate::types::SearchData<AttachmentsResult, Fields>;
pub type SearchResult = crate::types::SearchResult<AttachmentsResult, Fields>;

#[derive(Deserialize, Debug)]
pub struct Icon {
    pub key: String,
    pub name: String,
    pub icon: String,
}

#[derive(Deserialize, Debug)]
pub struct Color {
    pub key: String,
    pub name: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct Fields {
    pub name: String,
    pub slug: Option<String>,
    pub description: Option<String>,
    milestone: Option<JsonValue>,
    depth: u32,
    parent: Option<JsonValue>,
    pub icon: Icon,
    pub color: Color,
    #[serde(rename = "spacePHID")]
    pub space: Option<Phid>,
    #[serde(rename = "dateCreated", deserialize_with = "deserialize_timestamp")]
    pub created: DateTime<Utc>,
    #[serde(rename = "dateModified", deserialize_with = "deserialize_timestamp")]
    pub modified: DateTime<Utc>,
    policy: JsonValue,
}

#[derive(Deserialize, Debug)]
pub struct AttachmentsResult {}

impl ApiRequest for Search {
    type Reply = SearchResult;
    const ROUTE: &'static str = "api/project.search";
}

impl ApiRequest for SearchCursor<'_> {
    type Reply = SearchResult;
    const ROUTE: &'static str = "api/project.search";
}

#[cfg(test)]
mod test {
    use super::*;
    use phabricator_mock::project::Project;
    use phabricator_mock::PhabMockServer;

    fn compare_project(mock: &Project, response: &SearchData) {
        assert_eq!(mock.id, response.id);
        assert_eq!(mock.phid, response.phid.0.as_str());
    }

    #[tokio::test]
    async fn simple() {
        let m = PhabMockServer::start().await;
        let project = phabricator_mock::project()
            .id(25)
            .name("Project")
            .build()
            .unwrap();
        m.add_project(project.clone());

        let client = crate::Client::new(m.uri(), m.token().to_string());
        let s = Search {
            constraints: Constraints {
                ids: Some(vec![25]),
                ..Default::default()
            },
            ..Default::default()
        };

        let r = client.request(&s).await.unwrap();
        assert_eq!(r.data.len(), 1);

        assert_eq!(1, r.data.len());
        let project = r.data.iter().find(|d| d.id == 25).unwrap();
        let mock_project = m.get_project(25).unwrap();

        compare_project(&mock_project, project);
    }

    #[tokio::test]
    async fn no_result() {
        let m = PhabMockServer::start().await;
        let client = crate::Client::new(m.uri(), m.token().to_string());

        let s = Search {
            constraints: Constraints {
                ids: Some(vec![100, 200]),
                ..Default::default()
            },
            ..Default::default()
        };

        let r = client.request(&s).await.unwrap();
        assert_eq!(0, r.data.len());
    }
}
