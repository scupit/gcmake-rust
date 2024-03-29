use std::collections::{HashMap, HashSet, BTreeMap};

use colored::Colorize;

use crate::project_info::raw_data_in::dependencies::{internal_dep_config::{RawModuleDep, CMakeModuleType, raw_dep_common::{RawPredepCommon, RawEmscriptenConfig}}, user_given_dep_config::UserGivenPredefinedDependencyConfig};

use super::{predep_module_common::{PredefinedDepFunctionality, FinalDebianPackagesConfig, FinalDepConfigOption, resolve_final_config_options}, final_target_map_common::{FinalTargetConfigMap, make_final_target_config_map}};

#[derive(Clone)]
pub struct PredefinedCMakeModuleDep {
  raw_dep: RawModuleDep,
  target_map: FinalTargetConfigMap,
  debian_packages: FinalDebianPackagesConfig,
  cmake_namespaced_target_map: HashMap<String, String>,
  yaml_namespaced_target_map: HashMap<String, String>,
  config_options: BTreeMap<String, FinalDepConfigOption>,
  _can_cross_compile: bool
}

impl PredefinedCMakeModuleDep {
  pub fn get_gcmake_readme_link(&self) -> &str {
    &self.raw_dep.links.gcmake_readme
  }

  pub fn get_target_config_map(&self) -> &FinalTargetConfigMap {
    &self.target_map
  }

  pub fn find_module_base_name(&self) -> &str {
    &self.raw_dep.module_name
  }

  pub fn module_type(&self) -> &CMakeModuleType {
    &self.raw_dep.module_type
  }

  pub fn found_varname(&self) -> &str {
    &self.raw_dep.found_var
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
    user_given_dep_config: &UserGivenPredefinedDependencyConfig,
    dep_name: &str,
    valid_feature_list: Option<&Vec<&str>>
  ) -> Result<Self, String> {
    let target_map = make_final_target_config_map(dep_name, dep, valid_feature_list)
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
      debian_packages: FinalDebianPackagesConfig::make_from(dep.raw_debian_packages_config()),
      cmake_namespaced_target_map,
      yaml_namespaced_target_map,
      _can_cross_compile: dep.can_trivially_cross_compile(),
      config_options: resolve_final_config_options(
        dep.config_options_map(),
        user_given_dep_config.options.clone()
      )
        .map_err(|err_msg| format!(
          "In configuration for predefined dependency '{}':\n{}",
          dep_name.yellow(),
          err_msg
        ))?
    });
  }
}

impl PredefinedDepFunctionality for PredefinedCMakeModuleDep {
  fn debian_packages_config(&self) -> &FinalDebianPackagesConfig {
    &self.debian_packages
  }

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

  fn config_options_map(&self) -> &BTreeMap<String, FinalDepConfigOption> {
    &self.config_options
  }
}