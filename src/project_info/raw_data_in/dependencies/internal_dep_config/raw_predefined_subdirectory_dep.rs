use std::collections::{HashMap, HashSet};

use serde::{Deserialize};

use super::{raw_target_config_common::RawPredefinedTargetMapIn, RawMutualExclusionSet, raw_dep_common::RawPredepCommon};

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NamespaceConfig {
  pub cmakelists_linking: String
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GitRepoConfig {
  pub repo_url: String
}

fn default_requires_custom_populate() -> bool { false }

// A predefined dependency which exists within the project build tree.
// These should always be inside the dep/ folder.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawSubdirectoryDependency {
  pub namespace_config: NamespaceConfig,
  // Name of the include directory the library will be installed to. Usually this is the same as the
  // project name, but not always. For example, nlohmann_json installs its package config to the
  // nlohmann_json directory, but uses 'nlohmann' as its include dir.
  pub installed_include_dir_name: Option<String>,
  // Name of the directory the project is installed to. The directory name should be the name
  // of the project the 
  pub config_file_project_name: Option<String>,
  pub git_repo: GitRepoConfig,
  pub target_configs: RawPredefinedTargetMapIn,
  pub mutually_exclusive: Option<RawMutualExclusionSet>,
  #[serde(default = "default_requires_custom_populate")]
  pub requires_custom_fetchcontent_populate: bool
}

impl RawPredepCommon for RawSubdirectoryDependency {
  fn maybe_mutual_exclusion_groups(&self) -> &Option<RawMutualExclusionSet> {
    &self.mutually_exclusive
  }

  fn raw_target_map_in(&self) -> &RawPredefinedTargetMapIn {
    &self.target_configs
  }
}
