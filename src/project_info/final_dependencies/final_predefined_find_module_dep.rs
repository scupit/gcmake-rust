use std::collections::HashMap;

use crate::project_info::raw_data_in::dependencies::{internal_dep_config::RawBuiltinFindModuleDep, user_given_dep_config::UserGivenPredefinedDependencyConfig};

pub struct PredefinedFindModuleDep {
  raw_dep: RawBuiltinFindModuleDep,
  namespaced_target_map: HashMap<String, String>
}

impl PredefinedFindModuleDep {
  pub fn found_varname(&self) -> &str {
    &self.raw_dep.found_var
  }

  pub fn has_target_named(&self, target_name: &str) -> bool {
    self.namespaced_target_map.contains_key(target_name)
  }

  pub fn get_linkable_target_name(&self, target_name: &str) -> Option<&str> {
    self.namespaced_target_map.get(target_name)
      .map(|the_string| &the_string[..])
  }

  pub fn from_find_module_dep(
    dep: &RawBuiltinFindModuleDep,
    _user_given_dep_config: &UserGivenPredefinedDependencyConfig,
    _dep_name: &str
  ) -> Self {
    Self {
      raw_dep: dep.clone(),
      namespaced_target_map: dep.targets
        .iter()
        .map(|target_name| {
          (
            target_name.clone(),
            dep.namespaced_target(target_name).unwrap()
          )
        })
        .collect()
    }
  }
}