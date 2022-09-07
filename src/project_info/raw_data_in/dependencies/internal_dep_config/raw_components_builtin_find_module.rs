use std::collections::HashSet;

use serde::{Serialize, Deserialize};

use super::{CMakeModuleType, raw_target_config_common::RawPredefinedTargetMapIn, RawMutualExclusionSet, raw_dep_common::RawPredepCommon};

#[derive(Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct InstallationLinks {
  pub prebuilt_downloads: Option<String>,
  pub building: Option<String>
}

#[derive(Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct ComponentsFindModuleLinks {
  pub cmake_find_module: String,
  pub installation: Option<InstallationLinks>,
  pub components_doc: Option<String>
}

#[derive(Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub enum UsageMode {
  Variable,
  Target
}

#[derive(Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct ComponentsFindModuleUsage {
  pub link_format: UsageMode,
  pub link_value: String,
  pub found_var: String
}

#[derive(Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct RawComponentsModuleDep {
  pub links: ComponentsFindModuleLinks,
  pub module_type: CMakeModuleType,
  pub cmakelists_usage: ComponentsFindModuleUsage,
  pub mutually_exclusive: Option<RawMutualExclusionSet>,
  pub components: RawPredefinedTargetMapIn
}

impl RawPredepCommon for RawComponentsModuleDep {
  fn can_cross_compile(&self) -> bool {
    false
  }

  fn maybe_mutual_exclusion_groups(&self) -> &Option<RawMutualExclusionSet> {
    &self.mutually_exclusive
  }

  fn raw_target_map_in(&self) -> &RawPredefinedTargetMapIn {
    &self.components
  }

  fn repo_url(&self) -> Option<&str> {
    None
  }
}