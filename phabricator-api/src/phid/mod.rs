use super::types::Phid;
use serde::Deserialize;

mod query;
pub use query::*;

mod lookup;
pub use lookup::*;

#[derive(Deserialize, Debug)]
pub struct Item {
    #[serde(rename = "fullName")]
    full_name: String,
    name: String,
    phid: Phid,
    status: String,
    uri: String,
    #[serde(rename = "type")]
    ty: String,
    #[serde(rename = "typeName")]
    type_name: String,
}
