use serde::{Deserialize};

use super::{ComponentsFindModuleLinks, raw_target_config_common::RawPredefinedTargetMapIn, RawMutualExclusionSet, raw_dep_common::RawPredepCommon};

#[derive(Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct BuiltinFindModuleNamespaceConfig {
  pub cmakelists_linking: String
}

#[derive(Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub enum CMakeModuleType {
  ConfigFile,
  FindModule
}

#[derive(Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct RawModuleDep {
  pub found_var: String,
  pub module_type: CMakeModuleType,
  pub links: ComponentsFindModuleLinks,
  pub namespace_config: BuiltinFindModuleNamespaceConfig,
  pub mutually_exclusive: Option<RawMutualExclusionSet>,
  pub targets: RawPredefinedTargetMapIn
}

impl RawPredepCommon for RawModuleDep {
  fn can_cross_compile(&self) -> bool {
    false
  }

  fn maybe_mutual_exclusion_groups(&self) -> &Option<RawMutualExclusionSet> {
    &self.mutually_exclusive
  }

  fn raw_target_map_in(&self) -> &RawPredefinedTargetMapIn {
    &self.targets
  }
}
