use serde::{Deserialize};

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NamespaceConfig {
  cmakelists_linking: String
}

fn default_recursive_clone() -> bool { true }

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GitRepoConfig {
  #[serde(default = "default_recursive_clone")]
  pub recursive_clone: bool,
  pub repo_url: String
}

fn default_requires_custom_populate() -> bool { false }

// A predefined dependency which exists within the project build tree.
// These should always be inside the dep/ folder.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawSubdirectoryDependency {
  namespace_config: NamespaceConfig,
  // Name of the include directory the library will be installed to. Usually this is the same as the
  // project name, but not always. For example, nlohmann_json installs its package config to the
  // nlohmann_json directory, but uses 'nlohmann' as its include dir.
  pub installed_include_dir_name: Option<String>,
  // Name of the directory the project is installed to. The directory name should be the name
  // of the project the 
  pub config_file_project_name: Option<String>,
  pub git_repo: GitRepoConfig,
  pub target_names: Vec<String>,
  #[serde(default = "default_requires_custom_populate")]
  pub requires_custom_fetchcontent_populate: bool
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
