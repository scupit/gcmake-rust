use std::collections::HashSet;

use crate::project_info::raw_data_in::dependencies::{internal_dep_config::{RawComponentsModuleDep, ComponentsFindModuleLinks, UsageMode, CMakeModuleType}, user_given_dep_config::{UserGivenPredefinedDependencyConfig}};

use super::{predep_module_common::PredefinedDepFunctionality, final_target_map_common::{FinalTargetConfigMap, make_final_target_config_map}};

#[derive(Clone)]
pub struct PredefinedCMakeComponentsModuleDep {
  raw_dep: RawComponentsModuleDep,
  lib_link_mode: UsageMode,
  components: FinalTargetConfigMap
}

impl PredefinedCMakeComponentsModuleDep {
  pub fn module_type(&self) -> &CMakeModuleType {
    &self.raw_dep.module_type
  }

  pub fn web_links(&self) -> &ComponentsFindModuleLinks {
    &self.raw_dep.links
  }

  pub fn has_component_named(&self, name_searching: &str) -> bool {
    return self.components.contains_key(name_searching);
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

  pub fn linkable_string(&self, target_name: &str) -> Option<String> {
    match &self.raw_dep.cmakelists_usage.link_format {
      UsageMode::Target => {
        if self.components.contains_key(target_name) {
          let target_namespace: &str = &self.raw_dep.cmakelists_usage.link_value;
          Some(format!(
            "{}{}",
            target_namespace,
            target_name
          ))
        }
        else {
          None
        }
      },
      UsageMode::Variable => Some(format!(
        "${{{}}}",
        &self.raw_dep.cmakelists_usage.link_value
      ))
    }
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

    return Ok(Self {
      components,
      lib_link_mode: components_dep.cmakelists_usage.link_format.clone(),
      raw_dep: components_dep.clone()
    });
  }
}

impl PredefinedDepFunctionality for PredefinedCMakeComponentsModuleDep {
  fn get_target_config_map(&self) -> &FinalTargetConfigMap {
    &self.components
  }

  fn target_name_set(&self) -> HashSet<String> {
    return self.components.keys()
      .map(|key_string| key_string.clone())
      .collect()
  }
}