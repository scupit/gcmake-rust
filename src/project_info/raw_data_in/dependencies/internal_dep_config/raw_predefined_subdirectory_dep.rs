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

fn default_requires_custom_populate() -> bool { false }
fn default_install_if_linked() -> bool { false }

// A predefined dependency which exists within the project build tree.
// These should always be inside the dep/ folder.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawSubdirectoryDependency {
  namespace_config: NamespaceConfig,
  pub git_repo: GitRepoConfig,
  pub target_names: Vec<String>,
  #[serde(default = "default_requires_custom_populate")]
  pub requires_custom_fetchcontent_populate: bool,

  // When set to true, any target in the dependency linked to an output
  // will be added to the "additional_install_no_exports" target list
  // using "add_to_install_no_export_list(...)".
  #[serde(default = "default_install_if_linked")]
  pub install_if_linked: bool
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
