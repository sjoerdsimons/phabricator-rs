use derive_builder::Builder;

#[derive(Clone, Debug, Builder)]
pub struct Priority {
    pub value: u32,
    #[builder(setter(into))]
    pub name: String,
    #[builder(setter(into))]
    pub color: String,
}
