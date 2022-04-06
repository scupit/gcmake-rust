use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct UserGivenPredefinedDependencyConfig {
  pub git_tag: Option<String>,
  pub commit_hash: Option<String>
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct UserGivenGCMakeProjectDependency {
  pub git_tag: Option<String>,
  pub commit_hash: Option<String>,
  pub repo_url: String
}
