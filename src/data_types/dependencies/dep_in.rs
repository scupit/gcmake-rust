use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct RawPredefinedDependency {
  pub git_tag: Option<String>
}
