use super::str_or_u32;
use crate::types::Phid;
use crate::ApiRequest;
use chrono::DateTime;
use chrono::TimeZone;
use chrono::Utc;
use serde::de::Deserializer;
use serde::Deserialize;
use serde::Serialize;
use serde_json::value::Value as JsonValue;
use std::collections::HashMap;

fn deserialize_date<'de, D>(d: D) -> Result<DateTime<Utc>, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(d)?;
    let date: i64 = s.parse().map_err(serde::de::Error::custom)?;

    Ok(Utc.timestamp(date, 0))
}

#[derive(Serialize, Debug)]
pub struct Info {
    pub task_id: u32,
}

#[derive(Deserialize, Debug)]
pub struct InfoResult {
    #[serde(rename = "objectName")]
    name: String,
    #[serde(rename = "authorPHID")]
    author: Phid,
    #[serde(rename = "ccPHIDs")]
    cc: Vec<Phid>,
    title: String,
    description: String,
    uri: String,
    #[serde(deserialize_with = "str_or_u32")]
    id: u32,
    #[serde(rename = "isClosed")]
    closed: bool,
    phid: Phid,
    #[serde(rename = "ownerPHID")]
    owner: Option<Phid>,
    priority: String,
    #[serde(rename = "projectPHIDs")]
    projects: Vec<Phid>,
    auxiliary: HashMap<String, Option<JsonValue>>,
    status: String,
    #[serde(rename = "statusName")]
    status_name: String,
    #[serde(rename = "priorityColor")]
    priority_color: String,
    #[serde(rename = "dependsOnTaskPHIDs")]
    depends_on: Vec<Phid>,
    #[serde(rename = "dateCreated", deserialize_with = "deserialize_date")]
    created: DateTime<Utc>,
    #[serde(rename = "dateModified", deserialize_with = "deserialize_date")]
    modified: DateTime<Utc>,
    #[serde(flatten)]
    unparsed: serde_json::value::Value,
}

impl ApiRequest for Info {
    type Reply = InfoResult;
    const ROUTE: &'static str = "api/maniphest.info";
}

#[cfg(test)]
mod test {
    use super::*;
    use phabricator_mock::task::Task;
    use phabricator_mock::PhabMockServer;

    fn compare_task(server: &Task, response: &InfoResult) {
        assert_eq!(server.id, response.id);
        assert_eq!(server.full_name, response.title);
        assert_eq!(server.description, response.description);
        // TODO compare more fields
    }

    #[tokio::test]
    async fn simple() {
        let m = PhabMockServer::start().await;
        let user = m.new_user("user", "Test User");
        m.new_simple_task(100, &user);

        let client = crate::Client::new(m.uri(), m.token().to_string());
        let i = Info { task_id: 100 };

        let r = client.request(&i).await.unwrap();

        let server_task = m.get_task(100).unwrap();

        compare_task(&server_task, &r);
    }

    #[tokio::test]
    async fn no_result() {
        let m = PhabMockServer::start().await;

        let client = crate::Client::new(m.uri(), m.token().to_string());
        let i = Info { task_id: 100 };

        client.request(&i).await.unwrap_err();
    }
}
