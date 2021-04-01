use derive_builder::Builder;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Special {
    Default,
    Duplicate,
    Closed,
}

#[derive(Builder, Clone, Debug)]
pub struct Status {
    #[builder(setter(into))]
    pub value: String,
    #[builder(setter(into))]
    pub name: String,
    #[builder(setter(strip_option, into), default)]
    pub color: Option<String>,
    #[builder(setter(strip_option), default)]
    pub special: Option<Special>,
    #[builder(default)]
    pub closed: bool,
}
