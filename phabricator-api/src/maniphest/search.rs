use crate::types::Phid;
use crate::utils::{deserialize_timestamp, deserialize_timestamp_option};
use crate::ApiRequest;
use chrono::DateTime;
use chrono::Utc;
use rust_decimal::prelude::*;
use serde::de::Deserializer;
use serde::Deserialize;
use serde::Serialize;
use serde_json::value::Value as JsonValue;
use std::collections::HashMap;
use std::default::Default;
use std::ops::Not;

fn deserialize_raw_string<'de, D>(d: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    #[derive(Deserialize)]
    struct Raw {
        raw: String,
    }
    let r = Raw::deserialize(d)?;
    Ok(r.raw)
}

pub trait Serializable: erased_serde::Serialize + std::fmt::Debug {}
impl<T: erased_serde::Serialize + std::fmt::Debug> Serializable for T {}
erased_serde::serialize_trait_object!(Serializable);

#[derive(Serialize, Debug, Default)]
pub struct Constraints {
    pub ids: Option<Vec<u32>>,
    pub phids: Option<Vec<Phid>>,
    pub query: Option<String>,
    pub projects: Option<Vec<String>>,
    #[serde(flatten)]
    pub custom: Option<HashMap<String, Box<dyn Serializable + Send + Sync>>>,
}

#[derive(Serialize, Debug, Default)]
pub struct Attachments {
    #[serde(skip_serializing_if = "<&bool>::not")]
    pub subscribers: bool,
    #[serde(skip_serializing_if = "<&bool>::not")]
    pub columns: bool,
    #[serde(skip_serializing_if = "<&bool>::not")]
    pub projects: bool,
}

pub type Search = crate::types::Search<Attachments, Constraints>;
pub type SearchCursor<'a> = crate::types::SearchCursor<'a, Attachments, Constraints>;

pub type SearchData = crate::types::SearchData<AttachmentsResult, Fields>;
pub type SearchResult = crate::types::SearchResult<AttachmentsResult, Fields>;

#[derive(Deserialize, Debug)]
pub struct Status {
    pub value: String,
    pub name: String,
    pub color: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct Priority {
    pub value: u32,
    pub name: String,
    pub color: String,
}

#[derive(Deserialize, Debug)]
pub struct Fields {
    pub name: String,
    #[serde(deserialize_with = "deserialize_raw_string")]
    pub description: String,
    #[serde(rename = "authorPHID")]
    pub author_phid: Phid,
    #[serde(rename = "ownerPHID")]
    pub owner_phid: Option<Phid>,
    #[serde(rename = "closerPHID")]
    pub closer_phid: Option<Phid>,
    pub status: Status,
    pub priority: Priority,
    pub points: Option<Decimal>,
    #[serde(rename = "dateCreated", deserialize_with = "deserialize_timestamp")]
    pub created: DateTime<Utc>,
    #[serde(rename = "dateModified", deserialize_with = "deserialize_timestamp")]
    pub modified: DateTime<Utc>,
    #[serde(
        rename = "dateClosed",
        deserialize_with = "deserialize_timestamp_option"
    )]
    pub closed: Option<DateTime<Utc>>,
    policy: JsonValue,
}

#[derive(Deserialize, Debug)]
pub struct Subscriber {
    #[serde(rename = "subscriberCount")]
    pub count: u32,
    #[serde(rename = "subscriberPHIDs")]
    pub phids: Vec<Phid>,
    #[serde(rename = "viewerIsSubscribed")]
    pub viewer_is_subscribed: bool,
}

#[derive(Deserialize, Debug)]
pub struct Projects {
    #[serde(rename = "projectPHIDs")]
    pub projects: Vec<Phid>,
}

#[derive(Deserialize, Debug)]
pub struct Column {
    pub id: u32,
    pub name: String,
    pub phid: Phid,
}

#[derive(Deserialize, Debug)]
pub struct Columns {
    pub columns: Vec<Column>,
}

#[derive(Deserialize, Debug)]
pub struct Boards {
    pub boards: HashMap<Phid, Columns>,
}

#[derive(Deserialize, Debug)]
pub struct AttachmentsResult {
    pub columns: Option<Boards>,
    pub subscribers: Option<Subscriber>,
    pub projects: Option<Projects>,
}

impl ApiRequest for Search {
    type Reply = SearchResult;
    const ROUTE: &'static str = "api/maniphest.search";
}

impl ApiRequest for SearchCursor<'_> {
    type Reply = SearchResult;
    const ROUTE: &'static str = "api/maniphest.search";
}

#[cfg(test)]
mod test {
    use super::*;
    use phabricator_mock::task::Task;
    use phabricator_mock::PhabMockServer;

    fn compare_task(server: &Task, response: &SearchData) {
        assert_eq!(server.id, response.id);
        assert_eq!(server.full_name, response.fields.name);
        assert_eq!(server.description, response.fields.description);
        assert_eq!(server.points, response.fields.points);
        // TODO compare more fields
    }

    #[tokio::test]
    async fn basics() {
        let m = PhabMockServer::start().await;
        let user = m.new_user("user", "Test User");
        m.new_simple_task(100, &user);
        let status = m.new_status("foo", "bar", None);
        let task = phabricator_mock::task()
            .id(200)
            .full_name("Test task")
            .description("Test description")
            .author(user.clone())
            .owner(user.clone())
            .priority(m.default_priority())
            .status(status)
            .build()
            .unwrap();
        m.add_task(task);

        let client = crate::Client::new(m.uri(), m.token().to_string());
        let s = Search {
            constraints: Constraints {
                ids: Some(vec![100, 200]),
                ..Default::default()
            },
            ..Default::default()
        };

        let r = client.request(&s).await.unwrap();
        assert_eq!(r.data.len(), 2);

        let respond_task = r.data.iter().find(|d| d.id == 100).unwrap();
        let server_task = m.get_task(100).unwrap();

        compare_task(&server_task, respond_task);

        let respond_task = r.data.iter().find(|d| d.id == 200).unwrap();
        let server_task = m.get_task(200).unwrap();
        compare_task(&server_task, respond_task);
    }

    #[tokio::test]
    async fn subscribers() {
        let m = PhabMockServer::start().await;
        let user = m.new_user("user", "Test User");
        let subscriber = m.new_user("subscriber", "Subscribed usr");

        let task = phabricator_mock::task()
            .id(100)
            .full_name("Test task")
            .description("Test description")
            .author(user.clone())
            .owner(user.clone())
            .priority(m.default_priority())
            .status(m.default_status())
            .subscribers(vec![subscriber.clone()])
            .build()
            .unwrap();
        m.add_task(task);

        let client = crate::Client::new(m.uri(), m.token().to_string());
        let s = Search {
            constraints: Constraints {
                ids: Some(vec![100]),
                ..Default::default()
            },
            attachments: Attachments {
                subscribers: true,
                ..Default::default()
            },
            ..Default::default()
        };

        let r = client.request(&s).await.unwrap();
        assert_eq!(r.data.len(), 1);

        let respond_task = r.data.iter().find(|d| d.id == 100).unwrap();
        let server_task = m.get_task(100).unwrap();

        compare_task(&server_task, respond_task);

        let s = respond_task
            .attachments
            .subscribers
            .as_ref()
            .expect("No subscribers");
        assert_eq!(1, s.count);
        assert_eq!(1, s.phids.len());
        assert_eq!(&subscriber.phid.to_string(), &s.phids[0].0);
    }

    #[tokio::test]
    async fn projects() {
        let m = PhabMockServer::start().await;
        let user = m.new_user("user", "Test User");
        let project = phabricator_mock::project()
            .id(25)
            .name("Project")
            .build()
            .unwrap();
        m.add_project(project.clone());

        let task = phabricator_mock::task()
            .id(100)
            .full_name("Test task")
            .description("Test description")
            .author(user.clone())
            .owner(user.clone())
            .priority(m.default_priority())
            .status(m.default_status())
            .projects(vec![project.clone()])
            .build()
            .unwrap();
        m.add_task(task);

        let client = crate::Client::new(m.uri(), m.token().to_string());
        let s = Search {
            constraints: Constraints {
                ids: Some(vec![100]),
                ..Default::default()
            },
            attachments: Attachments {
                projects: true,
                ..Default::default()
            },
            ..Default::default()
        };

        let r = client.request(&s).await.unwrap();
        assert_eq!(r.data.len(), 1);

        let respond_task = r.data.iter().find(|d| d.id == 100).unwrap();
        let server_task = m.get_task(100).unwrap();

        compare_task(&server_task, respond_task);

        let p = respond_task
            .attachments
            .projects
            .as_ref()
            .expect("No projects");
        assert_eq!(1, p.projects.len());
        assert_eq!(&project.phid.to_string(), &p.projects[0].0);
    }

    #[tokio::test]
    async fn columns() {
        let m = PhabMockServer::start().await;
        let user = m.new_user("user", "Test User");
        let project = phabricator_mock::project()
            .id(25)
            .name("Project")
            .build()
            .unwrap();
        let column = phabricator_mock::column()
            .id(15)
            .name("Backlog")
            .project(project.clone())
            .build()
            .unwrap();
        project.add_column(column.clone());

        m.add_project(project.clone());

        let task = phabricator_mock::task()
            .id(100)
            .full_name("Test task")
            .description("Test description")
            .author(user.clone())
            .owner(user.clone())
            .priority(m.default_priority())
            .status(m.default_status())
            .columns(vec![column.clone()])
            .build()
            .unwrap();
        m.add_task(task);

        let client = crate::Client::new(m.uri(), m.token().to_string());
        let s = Search {
            constraints: Constraints {
                ids: Some(vec![100]),
                ..Default::default()
            },
            attachments: Attachments {
                columns: true,
                ..Default::default()
            },
            ..Default::default()
        };

        let r = client.request(&s).await.unwrap();
        assert_eq!(r.data.len(), 1);

        let respond_task = r.data.iter().find(|d| d.id == 100).unwrap();
        let server_task = m.get_task(100).unwrap();

        compare_task(&server_task, respond_task);

        let b = respond_task
            .attachments
            .columns
            .as_ref()
            .expect("No Columns");

        let c = &b.boards[&Phid(project.phid.to_string())];
        assert_eq!(1, c.columns.len());

        let c = &c.columns[0];
        assert_eq!(15, c.id);
        assert_eq!("Backlog", c.name);
        assert_eq!(column.phid.to_string(), c.phid.0);
    }

    #[tokio::test]
    async fn points() {
        let m = PhabMockServer::start().await;
        let user = m.new_user("user", "Test User");
        let task = phabricator_mock::task()
            .id(100)
            .points(Decimal::new(25, 2))
            .full_name("Test task")
            .description("Test description")
            .author(user.clone())
            .owner(user.clone())
            .priority(m.default_priority())
            .status(m.default_status())
            .build()
            .unwrap();
        m.add_task(task);

        let client = crate::Client::new(m.uri(), m.token().to_string());
        let s = Search {
            constraints: Constraints {
                ids: Some(vec![100]),
                ..Default::default()
            },
            ..Default::default()
        };

        let r = client.request(&s).await.unwrap();
        assert_eq!(r.data.len(), 1);

        let respond_task = r.data.iter().find(|d| d.id == 100).unwrap();
        let server_task = m.get_task(100).unwrap();

        compare_task(&server_task, respond_task);
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
