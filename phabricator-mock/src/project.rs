use crate::column::Column;
use crate::phid::Phid;
use crate::Policy;
use crate::Space;
use derive_builder::Builder;
use std::sync::{Arc, Mutex};

pub type Project = Arc<ProjectData>;

#[derive(Debug)]
pub struct ProjectIcon {
    pub key: String,
    pub name: String,
    pub icon: String,
}

impl Default for ProjectIcon {
    fn default() -> Self {
        ProjectIcon {
            key: "organization".to_string(),
            name: "Organization".to_string(),
            icon: "fa-building".to_string(),
        }
    }
}

#[derive(Debug)]
pub struct ProjectColor {
    pub key: String,
    pub name: Option<String>,
}

impl Default for ProjectColor {
    fn default() -> Self {
        ProjectColor {
            key: "disabled".to_string(),
            name: None,
        }
    }
}

#[derive(Debug, Default)]
pub struct ProjectPolicy {
    pub view: Policy,
    pub edit: Policy,
    pub join: Policy,
}

#[derive(Builder, Debug)]
#[builder(pattern = "owned", build_fn(name = "data_build"))]
pub struct ProjectData {
    pub id: u32,
    #[builder(setter(into))]
    pub name: String,
    #[builder(default, setter(into))]
    pub slug: Option<String>,
    #[builder(default)]
    pub description: Option<String>,
    #[builder(default)]
    pub icon: ProjectIcon,
    #[builder(default)]
    pub color: ProjectColor,
    #[builder(default = "Phid::new_project()")]
    pub phid: Phid,
    #[builder(default)]
    pub space: Option<Space>,
    #[builder(default)]
    pub created: u64,
    #[builder(default)]
    pub modified: u64,
    #[builder(default)]
    pub policy: ProjectPolicy,
    #[builder(default)]
    columns: Mutex<Vec<Column>>,
}

impl ProjectData {
    pub fn add_column(&self, column: Column) {
        let mut columns = self.columns.lock().unwrap();
        columns.push(column);
    }

    pub fn columns(&self) -> Vec<Column> {
        let columns = self.columns.lock().unwrap();
        columns.clone()
    }
}

impl ProjectDataBuilder {
    pub fn build(self) -> Result<Project, String> {
        self.data_build().map(Arc::new)
    }
}
