use std::collections::{HashMap, HashSet};

use crate::project_info::raw_data_in::dependencies::{internal_dep_config::{RawModuleDep, CMakeModuleType}, user_given_dep_config::UserGivenPredefinedDependencyConfig};

use super::{predep_module_common::PredefinedDepFunctionality, final_target_map_common::{FinalTargetConfigMap, make_final_target_config_map}};

#[derive(Clone)]
pub struct PredefinedCMakeModuleDep {
  raw_dep: RawModuleDep,
  target_map: FinalTargetConfigMap,
  namespaced_target_map: HashMap<String, String>
}

impl PredefinedCMakeModuleDep {
  pub fn get_target_config_map(&self) -> &FinalTargetConfigMap {
    &self.target_map
  }

  pub fn module_type(&self) -> &CMakeModuleType {
    &self.raw_dep.module_type
  }

  pub fn namespaced_target(&self, target_name: &str) -> Option<&str> {
    return self.namespaced_target_map.get(target_name)
      .map(|found_str| &found_str[..]);
  }

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
    dep: &RawModuleDep,
    _user_given_dep_config: &UserGivenPredefinedDependencyConfig,
    dep_name: &str
  ) -> Result<Self, String> {
    let target_map = make_final_target_config_map(
      dep_name,
      &dep.targets
    )
      .map_err(|err_msg| format!(
        "When loading predefined CMake Module dependency \"{}\": \n{}",
        dep_name,
        err_msg
      ))?;

    let mut namespaced_target_map: HashMap<String, String> = HashMap::new();

    for (target_name, _) in &dep.targets {
      namespaced_target_map.insert(
        target_name.to_string(),
        dep.namespaced_target(target_name).unwrap()
      );
    }

    return Ok(Self {
      raw_dep: dep.clone(),
      target_map,
      namespaced_target_map
    });
  }
}

impl PredefinedDepFunctionality for PredefinedCMakeModuleDep {
  fn get_target_config_map(&self) -> &FinalTargetConfigMap {
    &self.target_map
  }

  fn target_name_set(&self) -> HashSet<String> {
    self.namespaced_target_map.keys()
      .map(|k| k.to_string())
      .collect()
  }
}