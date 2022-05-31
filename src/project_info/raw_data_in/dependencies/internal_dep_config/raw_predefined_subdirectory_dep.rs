use serde::{Deserialize};

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NamespaceConfig {
  cmakelists_linking: String
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GitRepoConfig {
  pub repo_url: String
}

// A predefined dependency which exists within the project build tree.
// These should always be inside the dep/ folder.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawSubdirectoryDependency {
  namespace_config: NamespaceConfig,
  pub git_repo: GitRepoConfig,
  pub target_names: Vec<String>
}

impl RawSubdirectoryDependency {
  pub fn namespaced_target(&self, target_name: &str) -> Option<String> {
    for raw_target_name in &self.target_names {
      if raw_target_name == target_name {
        return Some(format!("{}{}", self.namespace_config.cmakelists_linking, target_name))
      }
    }
    None
  }
}
