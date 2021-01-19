use crate::phid::Phid;
use crate::Column;
use crate::Policy;
use crate::Priority;
use crate::Project;
use crate::Space;
use crate::Status;
use crate::User;
use derive_builder::Builder;
use rust_decimal::prelude::*;

use std::sync::Mutex;
use std::sync::{Arc, Weak};

#[derive(Clone, Debug, Default)]
pub struct TaskPolicy {
    pub view: Policy,
    pub interact: Policy,
    pub edit: Policy,
}

pub type Task = Arc<TaskData>;

#[derive(Builder)]
#[builder(pattern = "owned", build_fn(name = "data_build"), setter(strip_option))]
pub struct TaskData {
    pub id: u32,
    #[builder(setter(into))]
    pub full_name: String,
    #[builder(default = "Phid::new_task()")]
    pub phid: Phid,
    #[builder(setter(into))]
    pub description: String,
    pub author: User,
    #[builder(default)]
    pub owner: Option<User>,
    pub priority: Priority,
    #[builder(default)]
    pub points: Option<Decimal>,
    #[builder(default)]
    pub closer: Option<User>,
    pub status: Status,
    #[builder(default)]
    pub date_created: u64,
    #[builder(default)]
    pub date_modified: u64,
    #[builder(default)]
    pub date_closed: Option<u64>,
    #[builder(default)]
    pub space: Option<Space>,
    #[builder(default)]
    pub policy: TaskPolicy,
    #[builder(default)]
    pub projects: Vec<Project>,
    #[builder(default)]
    pub columns: Vec<Column>,
    #[builder(default)]
    pub subscribers: Vec<User>,
    #[builder(default)]
    parents: Mutex<Vec<Weak<TaskData>>>,
    #[builder(default)]
    subtasks: Mutex<Vec<Task>>,
}

impl TaskDataBuilder {
    pub fn build(self) -> Result<Task, String> {
        self.data_build().map(Arc::new)
    }
}

impl TaskData {
    pub fn parents(&self) -> Vec<Task> {
        let parents = self.parents.lock().unwrap();
        parents.iter().map(|t| t.upgrade().unwrap()).collect()
    }

    pub fn subtasks(&self) -> Vec<Task> {
        let subtasks = self.subtasks.lock().unwrap();
        subtasks.clone()
    }
}

pub fn link(parent: &Task, subtask: &Task) {
    let mut p = parent.subtasks.lock().unwrap();
    p.push(subtask.clone());

    let mut s = subtask.parents.lock().unwrap();
    s.push(Arc::downgrade(parent));
}
