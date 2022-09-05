use std::collections::{HashSet, HashMap};

use crate::project_info::raw_data_in::dependencies::{internal_dep_config::{RawComponentsModuleDep, ComponentsFindModuleLinks, UsageMode, CMakeModuleType, raw_dep_common::RawPredepCommon}, user_given_dep_config::{UserGivenPredefinedDependencyConfig}};

use super::{predep_module_common::PredefinedDepFunctionality, final_target_map_common::{FinalTargetConfigMap, make_final_target_config_map}};

#[derive(Clone)]
pub struct PredefinedCMakeComponentsModuleDep {
  raw_dep: RawComponentsModuleDep,
  lib_link_mode: UsageMode,
  cmake_namespaced_target_map: HashMap<String, String>,
  yaml_namespaced_target_map: HashMap<String, String>,
  components: FinalTargetConfigMap,
  _can_cross_compile: bool
}

impl PredefinedCMakeComponentsModuleDep {
  pub fn module_type(&self) -> &CMakeModuleType {
    &self.raw_dep.module_type
  }

  pub fn web_links(&self) -> &ComponentsFindModuleLinks {
    &self.raw_dep.links
  }

  pub fn found_varname(&self) -> &str {
    &self.raw_dep.cmakelists_usage.found_var
  }

  pub fn whole_lib_links_using_variable(&self) -> bool {
    return match &self.lib_link_mode {
      UsageMode::Variable => true,
      UsageMode::Target => false
    }
  }

  pub fn get_yaml_linkable_target_name(&self, target_name: &str) -> Option<&str> {
    return self.yaml_namespaced_target_map.get(target_name)
      .map(|the_str| &the_str[..]);
  }

  pub fn get_cmake_linkable_target_name(&self, target_name: &str) -> Option<&str> {
    return self.cmake_namespaced_target_map.get(target_name)
      .map(|the_str| &the_str[..]);
  }

  pub fn from_components_find_module_dep(
    components_dep: &RawComponentsModuleDep,
    _user_given_dep_config: &UserGivenPredefinedDependencyConfig,
    dep_name: &str
  ) -> Result<Self, String> {
    let components = make_final_target_config_map(dep_name, components_dep)
      .map_err(|err_msg| format!(
        "When loading predefined CMake Components Module dependency \"{}\":\n\n{}",
        dep_name,
        err_msg
      ))?;

    let mut cmake_namespaced_target_map: HashMap<String, String> = HashMap::new();

    for (target_name, target_config) in &components {
      let the_link_str: String = match &components_dep.cmakelists_usage.link_format {
        UsageMode::Variable => {
          format!(
            "${{{}}}",
            &components_dep.cmakelists_usage.link_value
          )
        },
        UsageMode::Target => {
          format!(
            "{}{}",
            &components_dep.cmakelists_usage.link_value,
            &target_config.cmakelists_name
          )
        }
      };

      cmake_namespaced_target_map.insert(
        target_name.to_string(),
        the_link_str
      );
    }

    let mut yaml_namespaced_target_map: HashMap<String, String> = HashMap::new();

    for (target_name, target_config) in &components {
      yaml_namespaced_target_map.insert(
        target_name.to_string(),
        format!(
          "{}::{}",
          dep_name.to_string(),
          target_config.cmakelists_name
        )
      );
    }

    return Ok(Self {
      components,
      cmake_namespaced_target_map,
      yaml_namespaced_target_map,
      lib_link_mode: components_dep.cmakelists_usage.link_format.clone(),
      raw_dep: components_dep.clone(),
      _can_cross_compile: components_dep.can_cross_compile()
    });
  }
}

impl PredefinedDepFunctionality for PredefinedCMakeComponentsModuleDep {
  fn can_cross_compile(&self) -> bool {
    self._can_cross_compile
  }

  fn get_target_config_map(&self) -> &FinalTargetConfigMap {
    &self.components
  }

  fn target_name_set(&self) -> HashSet<String> {
    return self.components.keys()
      .map(|key_string| key_string.clone())
      .collect()
  }
}