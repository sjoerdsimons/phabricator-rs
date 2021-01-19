use crate::phid::Phid;
use crate::project::Project;
use derive_builder::Builder;
use std::sync::Arc;

pub type Column = Arc<ColumnData>;

#[derive(Builder, Clone, Debug)]
#[builder(build_fn(name = "data_build"))]
pub struct ColumnData {
    pub project: Project,
    pub id: u32,
    #[builder(setter(into))]
    pub name: String,
    #[builder(default = "Phid::new_column()")]
    pub phid: Phid,
}

impl ColumnDataBuilder {
    pub fn build(&self) -> Result<Column, String> {
        self.data_build().map(Arc::new)
    }
}
