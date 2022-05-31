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
pub struct RawBuiltinFindModuleDep {
  pub found_var: String,
  pub links: ComponentsFindModuleLinks,
  namespace_config: BuiltinFindModuleNamespaceConfig,
  pub targets: HashSet<String>
}

impl RawBuiltinFindModuleDep {
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
