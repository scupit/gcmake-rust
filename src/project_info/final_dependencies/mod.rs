mod final_predefined_subdir_dep;
mod final_predefined_cmake_components_module_dep;
mod final_gcmake_project_dep;
mod final_predefined_cmake_module_dep;

use std::rc::Rc;

pub use final_predefined_subdir_dep::*;
pub use final_predefined_cmake_components_module_dep::*;
pub use final_gcmake_project_dep::*;
pub use final_predefined_cmake_module_dep::*;

use super::raw_data_in::dependencies::{internal_dep_config::{SingleRawPredefinedDependencyConfigGroup, AllRawPredefinedDependencies, RawPredefinedDependencyInfo, PredefinedCMakeDepHookFile}, user_given_dep_config::UserGivenPredefinedDependencyConfig};

type HookScriptContainer = Option<Rc<PredefinedCMakeDepHookFile>>;

pub enum FinalPredepInfo {
  Subdirectory(PredefinedSubdirDep),
  CMakeComponentsModule(PredefinedCMakeComponentsModuleDep),
  CMakeModule(PredefinedCMakeModuleDep)
}

pub struct FinalPredefinedDependencyConfig {
  predep_info: FinalPredepInfo,
  pre_load: HookScriptContainer,
  post_load: HookScriptContainer,
  custom_populate: HookScriptContainer
}

impl FinalPredefinedDependencyConfig {
  pub fn new(
    all_raw_dep_configs: &AllRawPredefinedDependencies,
    user_given_config: &UserGivenPredefinedDependencyConfig,
    dep_name: &str
  ) -> Result<Self, String> {
    let configs = PredefinedDependencyAllConfigs::new(
      all_raw_dep_configs,
      user_given_config,
      dep_name
    )?;

    let predep_info: FinalPredepInfo = if let Some(subdir_dep) = configs.as_subdirectory {
      // NOTE: Right now, a dependency which has both subproject and builtin_component_find_module
      // dependency configurations will have the subdirectory mode selected by default.
      // This is fine for now, but should be changed in the future.
      // For example, SFML is a component-based library which can be installed on the system.
      // and retrieved using CMake's find_package in CONFIG mode. However, SFML is currently
      // only configured for use as a subdirectory dependency.
      FinalPredepInfo::Subdirectory(subdir_dep)
    }
    else if let Some(find_module_dep) = configs.built_in_find_module {
      FinalPredepInfo::CMakeModule(find_module_dep)
    }
    else if let Some(components_find_module_dep) = configs.components_built_in_find_module {
      FinalPredepInfo::CMakeComponentsModule(components_find_module_dep)
    }
    else {
      return Err(format!(
        "Tried to use the predefined dependency '{}' which exists, but doesn't have a valid configuration.",
        dep_name
      ))
    };

    let RawPredefinedDependencyInfo {
      pre_load,
      post_load,
      custom_populate,
      ..
    } = all_raw_dep_configs.get(dep_name).unwrap(); 
    
    return Ok(Self {
      predep_info,
      pre_load: pre_load.clone(),
      post_load: post_load.clone(),
      custom_populate: custom_populate.clone()
    });
  }

  pub fn predefined_dep_info(&self) -> &FinalPredepInfo {
    &self.predep_info
  }

  pub fn mut_predef_dep_info(&mut self) -> &mut FinalPredepInfo {
    &mut self.predep_info
  }

  pub fn pre_load_script(&self) -> &HookScriptContainer {
    &self.pre_load
  }

  pub fn post_load_script(&self) -> &HookScriptContainer {
    &self.post_load
  }

  pub fn custom_populate_script(&self) -> &HookScriptContainer {
    &self.custom_populate
  }

  pub fn is_auto_fetchcontent_ready(&self) -> bool {
    match &self.predep_info {
      FinalPredepInfo::Subdirectory(subdir_info) => !subdir_info.requires_custom_fetchcontent_populate(),
      FinalPredepInfo::CMakeComponentsModule(_) => false,
      FinalPredepInfo::CMakeModule(_) => false
    }
  }

  pub fn should_install_if_linked(&self) -> bool {
    return match &self.predep_info {
      FinalPredepInfo::Subdirectory(subdir_info) => subdir_info.should_install_if_linked(),
      _ => false
    }
  }
}

struct PredefinedDependencyAllConfigs {
  as_subdirectory: Option<PredefinedSubdirDep>,
  components_built_in_find_module: Option<PredefinedCMakeComponentsModuleDep>,
  built_in_find_module: Option<PredefinedCMakeModuleDep>
}

impl PredefinedDependencyAllConfigs {
  pub fn new(
    all_raw_dep_configs: &AllRawPredefinedDependencies,
    user_given_config: &UserGivenPredefinedDependencyConfig,
    dep_name: &str
  ) -> Result<Self, String> {

    let dep_info: &RawPredefinedDependencyInfo = match all_raw_dep_configs.get(dep_name) {
      Some(info) => info,
      None => {
        return Err(format!(
          "Tried to use predefined dependency '{}' which doesn't exist. Make sure to double check capitalization and spelling.",
          dep_name
        ));
      }
    };

    let mut final_config: Self = Self {
      as_subdirectory: None,
      components_built_in_find_module: None,
      built_in_find_module: None
    };

    if let Some(subdir_dep_info) = &dep_info.dep_configs.as_subdirectory {
      final_config.as_subdirectory = Some(PredefinedSubdirDep::from_subdir_dep(
        subdir_dep_info,
        user_given_config,
        dep_name
      )?);
    }

    if let Some(components_find_module_dep) = &dep_info.dep_configs.cmake_components_module {
      final_config.components_built_in_find_module = Some(PredefinedCMakeComponentsModuleDep::from_components_find_module_dep(
        components_find_module_dep,
        user_given_config,
        dep_name
      ));
    }

    if let Some(find_module_dep_info) = &dep_info.dep_configs.cmake_module {
      final_config.built_in_find_module = Some(PredefinedCMakeModuleDep::from_find_module_dep(
        find_module_dep_info,
        user_given_config,
        dep_name
      ));
    }

    return Ok(final_config);
  }
}
