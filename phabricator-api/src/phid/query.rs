use crate::ApiRequest;
use serde::Deserialize;
use serde::Serialize;
use std::collections::HashMap;

use super::Item;

#[derive(Debug, Serialize)]
pub struct Query {
    pub phids: Vec<String>,
}

#[derive(Deserialize, Debug)]
#[serde(untagged)]
pub enum QueryResult {
    Results(HashMap<String, Item>),
    // If there are no results phabricator returns [] as the result....
    NoData([(); 0]),
}

impl ApiRequest for Query {
    type Reply = QueryResult;
    const ROUTE: &'static str = "api/phid.query";
}

#[cfg(test)]
mod test {
    use phabricator_mock::PhabMockServer;

    #[tokio::test]
    async fn basic() {
        let m = PhabMockServer::start().await;
        let _client = crate::Client::new(m.uri(), "".to_string());
    }
}
