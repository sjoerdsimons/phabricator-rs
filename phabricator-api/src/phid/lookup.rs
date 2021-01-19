use crate::ApiRequest;
use serde::de::Deserializer;
use serde::Deserialize;
use serde::Serialize;
use std::collections::HashMap;

use super::Item;

#[derive(Serialize)]
pub struct Lookup {
    pub names: Vec<String>,
}

fn data_or_no_data<'de, D>(deserializer: D) -> Result<HashMap<String, Item>, D::Error>
where
    D: Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum DorNd {
        Data(HashMap<String, Item>),
        NoData([(); 0]),
    }

    let d = DorNd::deserialize(deserializer)?;
    let v = match d {
        DorNd::Data(v) => v,
        DorNd::NoData(_) => HashMap::new(),
    };
    Ok(v)
}

#[derive(Deserialize, Debug)]
#[serde(transparent)]
pub struct LookupResult(#[serde(deserialize_with = "data_or_no_data")] HashMap<String, Item>);

impl ApiRequest for Lookup {
    type Reply = LookupResult;
    const ROUTE: &'static str = "api/phid.lookup";
}

#[cfg(test)]
mod test {
    use super::*;
    use phabricator_mock::PhabMockServer;

    #[tokio::test]
    async fn simple() {
        let m = PhabMockServer::start().await;

        let user = m.new_user("user", "Test User");
        m.new_simple_task(100, &user);

        let client = crate::Client::new(m.uri(), m.token().to_string());

        let l = Lookup {
            names: vec!["T100".to_owned()],
        };
        let r = client.request(&l).await.unwrap();

        assert!(r.0.contains_key("T100"));
    }

    #[tokio::test]
    async fn multiple() {
        let m = PhabMockServer::start().await;
        let user = m.new_user("user", "Test User");
        m.new_simple_task(100, &user);
        m.new_simple_task(200, &user);

        let client = crate::Client::new(m.uri(), m.token().to_string());

        let l = Lookup {
            names: vec!["T100".to_owned(), "T200".to_owned()],
        };
        let r = client.request(&l).await.unwrap();

        for n in l.names {
            assert!(r.0.contains_key(&n), "missing task {}", &n);
        }
    }

    #[tokio::test]
    async fn no_result() {
        let m = PhabMockServer::start().await;
        let client = crate::Client::new(m.uri(), m.token().to_string());

        let l = Lookup {
            names: vec!["T100".to_owned(), "T200".to_owned()],
        };
        let r = client.request(&l).await.unwrap();
        assert_eq!(r.0.len(), 0);
    }
}
