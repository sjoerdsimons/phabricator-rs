use crate::types::{Cursor, Phid};
use crate::ApiRequest;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, PartialEq, Eq)]
pub enum Type {
    #[serde(rename = "task.subtask")]
    TaskSubtask,
    #[serde(rename = "task.parent")]
    TaskParent,
}

#[derive(Serialize, Debug, Default)]
pub struct Search {
    #[serde(rename = "sourcePHIDs")]
    pub sources: Vec<Phid>,
    pub types: Vec<Type>,
    #[serde(rename = "destinationPHIDs")]
    pub destinations: Option<Vec<Phid>>,
}

#[derive(Serialize, Debug)]
pub struct SearchCursor<'a> {
    #[serde(flatten)]
    pub cursor: &'a Cursor,
    #[serde(flatten)]
    pub search: &'a Search,
}

#[derive(Deserialize, Debug)]
pub struct Data {
    #[serde(rename = "sourcePHID")]
    pub source: Phid,
    #[serde(rename = "edgeType")]
    pub ty: Type,
    #[serde(rename = "destinationPHID")]
    pub dest: Phid,
}

#[derive(Deserialize, Debug)]
pub struct SearchResult {
    pub data: Vec<Data>,
    pub cursor: Cursor,
}

impl ApiRequest for Search {
    type Reply = SearchResult;
    const ROUTE: &'static str = "api/edge.search";
}

impl ApiRequest for SearchCursor<'_> {
    type Reply = SearchResult;
    const ROUTE: &'static str = "api/edge.search";
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::types::Phid;
    use phabricator_mock::task;
    use phabricator_mock::PhabMockServer;

    #[tokio::test]
    async fn simple() {
        let m = PhabMockServer::start().await;
        let user = m.new_user("user", "Test User");
        let t100 = m.new_simple_task(100, &user);
        let t200 = m.new_simple_task(200, &user);

        task::link(&t100, &t200);

        let client = crate::Client::new(m.uri(), m.token().to_string());
        let s = Search {
            sources: vec![Phid(t100.phid.to_string())],
            types: vec![Type::TaskSubtask],
            ..Default::default()
        };

        let r = client.request(&s).await.unwrap();
        assert_eq!(r.data.len(), 1);
        assert_eq!(t100.phid, r.data[0].source.0.as_str());
        assert_eq!(t200.phid, r.data[0].dest.0.as_str());
        assert_eq!(Type::TaskSubtask, r.data[0].ty);

        let s = Search {
            sources: vec![Phid(t200.phid.to_string())],
            types: vec![Type::TaskParent],
            ..Default::default()
        };

        let r = client.request(&s).await.unwrap();
        assert_eq!(r.data.len(), 1);
        assert_eq!(t200.phid, r.data[0].source.0.as_str());
        assert_eq!(t100.phid, r.data[0].dest.0.as_str());
        assert_eq!(Type::TaskParent, r.data[0].ty);
    }
}
