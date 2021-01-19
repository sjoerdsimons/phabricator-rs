use serde::Deserialize;
use serde::Serialize;
use serde_json::value::Value as JsonValue;

use crate::utils::str_or_u32;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct Phid(pub String);

#[derive(Serialize, Deserialize, Debug)]
pub struct Cursor {
    pub before: Option<String>,
    pub after: Option<String>,
    #[serde(deserialize_with = "str_or_u32")]
    pub limit: u32,
    pub order: Option<String>,
}

#[derive(Serialize, Debug, Default)]
pub struct Search<A, C> {
    #[serde(rename = "queryKey")]
    pub query_key: Option<String>,
    pub constraints: C,
    pub attachments: A,
}

#[derive(Serialize, Debug)]
pub struct SearchCursor<'a, A, C> {
    #[serde(flatten)]
    pub cursor: &'a Cursor,
    #[serde(flatten)]
    pub search: &'a Search<A, C>,
}

#[derive(Deserialize, Debug)]
pub struct SearchData<A, F> {
    pub id: u32,
    #[serde(rename = "type")]
    pub ty: String,
    pub phid: Phid,
    pub fields: F,
    pub attachments: A,
}

#[derive(Deserialize, Debug)]
pub struct SearchResult<A, F> {
    pub cursor: Cursor,
    pub data: Vec<SearchData<A, F>>,
    #[serde(flatten)]
    unparsed: JsonValue,
}
