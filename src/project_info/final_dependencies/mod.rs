mod final_predefined_subdir_dep;
mod final_predefined_cmake_components_module_dep;
mod final_gcmake_project_dep;
mod final_predefined_cmake_module_dep;
mod predep_module_common;
mod final_target_map_common;

use std::{rc::Rc, collections::HashMap};

use base64ct::{Base64Url, Encoding};
pub use final_predefined_subdir_dep::*;
pub use final_predefined_cmake_components_module_dep::*;
pub use final_gcmake_project_dep::*;
pub use final_predefined_cmake_module_dep::*;
pub use final_target_map_common::{FinalRequirementSpecifier, FinalTargetConfig, FinalExternalRequirementSpecifier};
pub use predep_module_common::{PredefinedDepFunctionality, FinalDebianPackagesConfig};

use crate::project_info::{platform_spec_parser::parse_leading_system_spec, parsers::general_parser::ParseSuccess};

use self::{final_target_map_common::FinalTargetConfigMap};

use super::raw_data_in::dependencies::{internal_dep_config::{AllRawPredefinedDependencies, RawPredefinedDependencyInfo, PredefinedCMakeDepHookFile, RawSubdirectoryDependency, raw_dep_common::RawEmscriptenConfig, CMakeModuleType}, user_given_dep_config::UserGivenPredefinedDependencyConfig};

pub fn base64_encoded(some_url: &str) -> String{
  return Base64Url::encode_string(some_url.as_bytes());
}

type HookScriptContainer = Option<Rc<PredefinedCMakeDepHookFile>>;

#[derive(Clone)]
pub enum FinalPredepInfo {
  Subdirectory(PredefinedSubdirDep),
  CMakeComponentsModule(PredefinedCMakeComponentsModuleDep),
  CMakeModule(PredefinedCMakeModuleDep)
}

impl FinalPredepInfo {
  pub fn can_cross_compile(&self) -> bool {
    match self {
      Self::Subdirectory(subdir_dep) => subdir_dep.can_cross_compile(),
      Self::CMakeComponentsModule(components_dep) => components_dep.can_cross_compile(),
      Self::CMakeModule(module_dep) => module_dep.can_cross_compile()
    }
  }

  pub fn supports_emscripten(&self) -> bool {
    match self {
      Self::Subdirectory(subdir_dep) => subdir_dep.supports_emscripten(),
      Self::CMakeComponentsModule(components_dep) => components_dep.supports_emscripten(),
      Self::CMakeModule(module_dep) => module_dep.supports_emscripten()
    }
  }

  pub fn emscripten_config(&self) -> Option<&RawEmscriptenConfig> {
    match self {
      Self::Subdirectory(subdir_dep) => subdir_dep.raw_emscripten_config(),
      Self::CMakeComponentsModule(components_dep) => components_dep.raw_emscripten_config(),
      Self::CMakeModule(module_dep) => module_dep.raw_emscripten_config()
    }
  }

  pub fn is_internally_supported_by_emscripten(&self) -> bool {
    match self {
      Self::Subdirectory(subdir_dep) => subdir_dep.is_internally_supported_by_emscripten(),
      Self::CMakeComponentsModule(components_dep) => components_dep.is_internally_supported_by_emscripten(),
      Self::CMakeModule(module_dep) => module_dep.is_internally_supported_by_emscripten()
    }
  }

  pub fn target_config_map(&self) -> &HashMap<String, FinalTargetConfig> {
    match self {
      Self::Subdirectory(subdir_dep) => subdir_dep.get_target_config_map(),
      Self::CMakeComponentsModule(components_dep) => components_dep.get_target_config_map(),
      Self::CMakeModule(module_dep) => module_dep.get_target_config_map()
    }
  }
}

#[derive(Clone)]
pub struct FinalPredefinedDependencyConfig {
  name: String,
  predep_info: FinalPredepInfo,
  pre_load: HookScriptContainer,
  post_load: HookScriptContainer,
  custom_populate: HookScriptContainer,
  custom_find_module: HookScriptContainer
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

    for (target_name, target_config) in predep_info.target_config_map() {
      for external_requirement in &target_config.external_requirements_set {
        match external_requirement {
          FinalExternalRequirementSpecifier::OneOf(link_spec_list) => {
            // Each link specifier here is guaranteed to only contain a single namespace and a single library.
            for link_spec in link_spec_list {
              assert!(
                link_spec.get_target_list().len() == 1,
                "An external link specifier should be guaranteed to only specify one library."
              );

              assert!(
                link_spec.get_namespace_queue().len() == 1,
                "An external link specifier should be guaranteed to have no nested namespaces."
              );
            
              let the_namespace: &str = link_spec.get_namespace_queue().iter().nth(0).unwrap();
              let required_lib_name: &str = link_spec.get_target_list().iter().nth(0).unwrap().get_name();

              match all_raw_dep_configs.get(the_namespace) {
                None => {
                  return Err(format!(
                    "The external dependency '{}' of target '{}' requires a library from predefined dependency '{}', however there is no predefined dependency named '{}'.",
                    link_spec.original_spec_str(),
                    target_name,
                    the_namespace,
                    the_namespace
                  ));
                },
                Some(raw_predep_config) => {
                  let mut has_matching_target_name: bool = false;

                  for (unparsed_target_name, _) in raw_predep_config.dep_configs.get_common()?.raw_target_map_in() {
                    let raw_target_name: &str = match parse_leading_system_spec(unparsed_target_name)? {
                      None => unparsed_target_name,
                      Some(ParseSuccess { value: _, rest }) => rest
                    };

                    if raw_target_name == required_lib_name {
                      has_matching_target_name = true;
                      break;
                    }
                  }

                  if !has_matching_target_name {
                    return Err(format!(
                      "The external dependency '{}' of target '{}' requires a library from predefined dependency '{}', but '{}' doesn't have a target named '{}'",
                      link_spec.original_spec_str(),
                      target_name,
                      the_namespace,
                      the_namespace,
                      required_lib_name
                    ));
                  }
                }
              }
            }
          }
        }
      }
    }

    let RawPredefinedDependencyInfo {
      pre_load,
      post_load,
      custom_populate,
      custom_find_module,
      ..
    } = all_raw_dep_configs.get(dep_name).unwrap(); 

    match &predep_info {
      FinalPredepInfo::CMakeModule(module_dep) => match (module_dep.module_type(), custom_find_module) {
        (CMakeModuleType::CustomFindModule, None) => {
          return Err(format!(
            "Predefined dependency '{}' must have an associated custom \"Find Module\" cmake file, but one doesn't exist.",
            dep_name
          ));
        },
        (CMakeModuleType::BuiltinFindModule, Some(finder_file)) => {
          return Err(format!(
            "Predefined dependency '{}' has an associated custom \"Find Module\" cmake file '{}', but is configured as a \"BuildinFindModule\" dependency. To use the custom find module, change the configuration for '{}' to the \"CustomFindModule\" type.",
            dep_name,
            finder_file.file_path.file_name().unwrap().to_str().unwrap(),
            dep_name
          ));
        },
        _ => ()
      },
      _ => ()
    }
    
    return Ok(Self {
      name: dep_name.to_string(),
      predep_info,
      pre_load: pre_load.clone(),
      post_load: post_load.clone(),
      custom_populate: custom_populate.clone(),
      custom_find_module: custom_find_module.clone()
    });
  }

  pub fn can_trivially_cross_compile(&self) -> bool {
    self.predep_info.can_cross_compile()
  }

  pub fn supports_emscripten(&self) -> bool {
    self.predep_info.supports_emscripten()
  }

  pub fn emscripten_config(&self) -> Option<&RawEmscriptenConfig> {
    self.predep_info.emscripten_config()
  }

  pub fn get_name(&self) -> &str {
    &self.name
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

  pub fn custom_find_module_file(&self) -> &HookScriptContainer {
    &self.custom_find_module
  }

  pub fn is_fetchcontent(&self) -> bool {
    match &self.predep_info {
      FinalPredepInfo::Subdirectory(_) => true,
      FinalPredepInfo::CMakeComponentsModule(_) => false,
      FinalPredepInfo::CMakeModule(_) => false
    }
  }

  pub fn is_auto_fetchcontent_ready(&self) -> bool {
    match &self.predep_info {
      FinalPredepInfo::Subdirectory(subdir_info) => !subdir_info.requires_custom_fetchcontent_populate(),
      FinalPredepInfo::CMakeComponentsModule(_) => false,
      FinalPredepInfo::CMakeModule(_) => false
    }
  }

  pub fn should_install_if_linked_to_output_library(&self) -> bool {
    return match &self.predep_info {
      FinalPredepInfo::Subdirectory(_) => true,
      _ => false
    }
  }

  // pub fn target_name_set(&self) -> HashSet<String> {
  //   return self.unwrap_dep_common().target_name_set();
  // }

  pub fn get_target_config_map(&self) -> &FinalTargetConfigMap {
    return self.as_common().get_target_config_map();
  }

  pub fn get_cmake_namespaced_target_name(&self, target_name: &str) -> Option<String> {
    match self.predefined_dep_info() {
      FinalPredepInfo::Subdirectory(subdir_dep) =>
        subdir_dep.get_cmake_linkable_target_name(target_name).map(String::from),
      FinalPredepInfo::CMakeModule(module_dep) =>
        module_dep.get_cmake_linkable_target_name(target_name).map(String::from),
      FinalPredepInfo::CMakeComponentsModule(components_dep) =>
        components_dep.get_cmake_linkable_target_name(target_name).map(String::from)
    }
  }

  pub fn get_yaml_namespaced_target_name(&self, target_name: &str) -> Option<String> {
    match self.predefined_dep_info() {
      FinalPredepInfo::Subdirectory(subdir_dep) =>
        subdir_dep.get_yaml_linkable_target_name(target_name).map(String::from),
      FinalPredepInfo::CMakeModule(module_dep) =>
        module_dep.get_yaml_linkable_target_name(target_name).map(String::from),
      FinalPredepInfo::CMakeComponentsModule(components_dep) =>
        components_dep.get_yaml_linkable_target_name(target_name).map(String::from)
    }
  }

  pub fn as_common(&self) -> &dyn PredefinedDepFunctionality {
    let the_dep: &dyn PredefinedDepFunctionality = match &self.predep_info {
      FinalPredepInfo::Subdirectory(subdir_dep) => subdir_dep,
      FinalPredepInfo::CMakeModule(module_dep) => module_dep,
      FinalPredepInfo::CMakeComponentsModule(components_dep) => components_dep
    };

    return the_dep;
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
      let final_components_dep = PredefinedCMakeComponentsModuleDep::from_components_find_module_dep(
        components_find_module_dep,
        user_given_config,
        dep_name
      )?;

      final_config.components_built_in_find_module = Some(final_components_dep);
    }

    if let Some(find_module_dep_info) = &dep_info.dep_configs.cmake_module {
      let final_find_module_info = PredefinedCMakeModuleDep::from_find_module_dep(
        find_module_dep_info,
        user_given_config,
        dep_name
      )?;

      final_config.built_in_find_module = Some(final_find_module_info);
    }

    return Ok(final_config);
  }
}
