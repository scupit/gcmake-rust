use std::collections::HashSet;

use serde::{Deserialize};

use super::{ComponentsFindModuleLinks, target_config_common::RawPredefinedTargetMapIn, RawMutualExclusionSet, raw_dep_common::RawPredepCommon};

#[derive(Deserialize, Clone)]
#[serde(deny_unknown_fields)]
struct BuiltinFindModuleNamespaceConfig {
  cmakelists_linking: String
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
  namespace_config: BuiltinFindModuleNamespaceConfig,
  pub mutually_exclusive: Option<RawMutualExclusionSet>,
  pub targets: RawPredefinedTargetMapIn
}

impl RawModuleDep {
  pub fn namespaced_target(&self, target_name: &str) -> Option<String> {
    return if self.targets.contains_key(target_name) {
      Some(format!(
        "{}{}",
        &self.namespace_config.cmakelists_linking,
        target_name
      ))
    }
    else { None }
  }
}

impl RawPredepCommon for RawModuleDep {
  fn maybe_mutual_exclusion_groups(&self) -> &Option<RawMutualExclusionSet> {
    &self.mutually_exclusive
  }

  fn raw_target_map_in(&self) -> &RawPredefinedTargetMapIn {
    &self.targets
  }
}
