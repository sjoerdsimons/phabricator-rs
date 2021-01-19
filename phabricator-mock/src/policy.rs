use crate::Project;
use serde::{Serialize, Serializer};

#[derive(Clone, Debug)]
pub enum Policy {
    Project(Project),
    Keyword(String),
}

impl Default for Policy {
    fn default() -> Self {
        Policy::Keyword("users".to_string())
    }
}

impl Serialize for Policy {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            Self::Project(ref p) => p.phid.serialize(serializer),
            Self::Keyword(ref s) => serializer.serialize_str(s),
        }
    }
}
