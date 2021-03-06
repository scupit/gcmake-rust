use std::collections::HashSet;

use serde::{Deserialize};

use super::ComponentsFindModuleLinks;

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
  pub targets: HashSet<String>
}

impl RawModuleDep {
  pub fn namespaced_target(&self, target_name: &str) -> Option<String> {
    return if self.targets.contains(target_name) {
      Some(format!(
        "{}{}",
        &self.namespace_config.cmakelists_linking,
        target_name
      ))
    }
    else { None }
  }
}
