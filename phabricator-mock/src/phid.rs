use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use serde::{Serialize, Serializer};
use std::fmt;
use std::iter;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum PhidType {
    Column,
    Project,
    Task,
    User,
}

impl fmt::Display for PhidType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let t = match self {
            PhidType::Column => "PCOL",
            PhidType::Project => "PROJ",
            PhidType::Task => "TASK",
            PhidType::User => "USER",
        };
        write!(f, "{}", t)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Phid {
    ty: PhidType,
    id: String,
}

impl Serialize for Phid {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl PartialEq<&str> for Phid {
    fn eq(&self, other: &&str) -> bool {
        &self.to_string() == other
    }
}

impl PartialEq<Phid> for &str {
    fn eq(&self, other: &Phid) -> bool {
        self == &other.to_string()
    }
}

impl Phid {
    pub fn new(ty: PhidType) -> Self {
        let mut rng = thread_rng();

        let id: String = iter::repeat(())
            .map(|()| rng.sample(Alphanumeric))
            .map(char::from)
            .map(|c| char::to_ascii_lowercase(&c))
            .take(21)
            .collect();

        Phid { ty, id }
    }

    pub fn new_user() -> Self {
        Self::new(PhidType::User)
    }

    pub fn new_task() -> Self {
        Self::new(PhidType::Task)
    }

    pub fn new_project() -> Self {
        Self::new(PhidType::Project)
    }

    pub fn new_column() -> Self {
        Self::new(PhidType::Column)
    }
}

impl fmt::Display for Phid {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "PHID-{}-{}", self.ty, self.id)
    }
}

impl std::str::FromStr for Phid {
    type Err = (); //TODO error handling
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.strip_prefix("PHID-").ok_or(())?;
        let mut split = s.splitn(2, '-');
        let ty = match split.next().ok_or(())? {
            "PCOL" => PhidType::Column,
            "PROJ" => PhidType::Project,
            "TASK" => PhidType::Task,
            "USER" => PhidType::User,
            _ => return Err(()),
        };
        let id = split.next().ok_or(())?.to_string();

        Ok(Phid { ty, id })
    }
}
