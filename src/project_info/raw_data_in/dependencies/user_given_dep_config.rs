use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct UserGivenPredefinedDependencyConfig {
  pub git_tag: Option<String>,
  pub commit_hash: Option<String>
}
