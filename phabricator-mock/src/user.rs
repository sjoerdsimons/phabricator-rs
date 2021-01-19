use crate::phid::Phid;
use derive_builder::Builder;
use std::sync::Arc;

pub type User = Arc<UserData>;

#[derive(Builder)]
#[builder(build_fn(name = "data_build"))]
pub struct UserData {
    #[builder(setter(into))]
    pub full_name: String,
    #[builder(setter(into))]
    pub name: String,
    #[builder(default = "Phid::new_user()")]
    pub phid: Phid,
}

impl UserDataBuilder {
    pub fn build(&self) -> Result<User, String> {
        self.data_build().map(Arc::new)
    }
}
