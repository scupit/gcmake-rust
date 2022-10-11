use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub enum UserGivenDownloadMode {
  #[serde(rename = "url")]
  Url,
  #[serde(rename = "git")]
  Git
}

/*
predefined_dependencies:
  nlohmann_json:
    file_version: v3.11.2

predefined_dependencies:
  nlohmann_json:
    git_tag: v3.11.2
*/
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct UserGivenPredefinedDependencyConfig {
  // URL mode options
  pub file_version: Option<String>,

  // Git mode options
  pub git_tag: Option<String>,
  pub commit_hash: Option<String>,
  pub repo_url: Option<String>
}

impl UserGivenPredefinedDependencyConfig {
  pub fn specifies_url_mode_options(&self) -> bool {
    return self.file_version.is_some();
  }

  pub fn specifies_git_mode_options(&self) -> bool {
    return self.git_tag.is_some()
      || self.commit_hash.is_some()
      || self.repo_url.is_some();
  }
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct UserGivenGCMakeProjectDependency {
  pub git_tag: Option<String>,
  pub commit_hash: Option<String>,
  pub repo_url: String
}
