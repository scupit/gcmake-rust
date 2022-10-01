use std::collections::{HashMap, HashSet};

use crate::project_info::raw_data_in::dependencies::{internal_dep_config::{RawModuleDep, CMakeModuleType, raw_dep_common::{RawPredepCommon, RawEmscriptenConfig}}, user_given_dep_config::UserGivenPredefinedDependencyConfig};

use super::{predep_module_common::PredefinedDepFunctionality, final_target_map_common::{FinalTargetConfigMap, make_final_target_config_map}};

#[derive(Clone)]
pub struct PredefinedCMakeModuleDep {
  raw_dep: RawModuleDep,
  target_map: FinalTargetConfigMap,
  cmake_namespaced_target_map: HashMap<String, String>,
  yaml_namespaced_target_map: HashMap<String, String>,
  _can_cross_compile: bool
}

impl PredefinedCMakeModuleDep {
  pub fn get_target_config_map(&self) -> &FinalTargetConfigMap {
    &self.target_map
  }

  pub fn module_type(&self) -> &CMakeModuleType {
    &self.raw_dep.module_type
  }

  pub fn found_varname(&self) -> &str {
    &self.raw_dep.found_var
  }

  pub fn has_target_named(&self, target_name: &str) -> bool {
    self.cmake_namespaced_target_map.contains_key(target_name)
  }

  pub fn get_yaml_linkable_target_name(&self, target_name: &str) -> Option<&str> {
    self.yaml_namespaced_target_map.get(target_name)
      .map(|the_string| &the_string[..])
  }

  pub fn get_cmake_linkable_target_name(&self, target_name: &str) -> Option<&str> {
    self.cmake_namespaced_target_map.get(target_name)
      .map(|the_string| &the_string[..])
  }

  pub fn from_find_module_dep(
    dep: &RawModuleDep,
    _user_given_dep_config: &UserGivenPredefinedDependencyConfig,
    dep_name: &str
  ) -> Result<Self, String> {
    let target_map = make_final_target_config_map(dep_name, dep)
      .map_err(|err_msg| format!(
        "When loading predefined CMake Module dependency \"{}\": \n\n{}",
        dep_name,
        err_msg
      ))?;

    let mut cmake_namespaced_target_map: HashMap<String, String> = HashMap::new();

    for (target_name, target_config) in &target_map {
      cmake_namespaced_target_map.insert(
        target_name.to_string(),
        format!(
          "{}{}",
          &dep.namespace_config.cmakelists_linking,
          &target_config.cmakelists_name
        )
      );
    }

    let mut yaml_namespaced_target_map: HashMap<String, String> = HashMap::new();

    for (target_name, target_config) in &target_map {
      yaml_namespaced_target_map.insert(
        target_name.to_string(),
        format!(
          "{}::{}",
          dep_name,
          &target_config.cmake_yaml_name
        )
      );
    }

    return Ok(Self {
      raw_dep: dep.clone(),
      target_map,
      cmake_namespaced_target_map,
      yaml_namespaced_target_map,
      _can_cross_compile: dep.can_trivially_cross_compile()
    });
  }
}

impl PredefinedDepFunctionality for PredefinedCMakeModuleDep {
  fn can_cross_compile(&self) -> bool {
    self._can_cross_compile
  }

  fn get_target_config_map(&self) -> &FinalTargetConfigMap {
    &self.target_map
  }

  fn target_name_set(&self) -> HashSet<String> {
    self.cmake_namespaced_target_map.keys()
      .map(|k| k.to_string())
      .collect()
  }

  fn supports_emscripten(&self) -> bool {
    self.raw_dep.supports_emscripten()
  }

  fn raw_emscripten_config(&self) -> Option<&RawEmscriptenConfig> {
    self.raw_dep.get_emscripten_config()
  }

  fn uses_emscripten_link_flag(&self) -> bool {
    match self.raw_emscripten_config() {
      None => false,
      Some(config) => config.link_flag.is_some()
    }
  }

  fn is_internally_supported_by_emscripten(&self) -> bool {
    self.raw_dep.is_internally_supported_by_emscripten()
  }
}