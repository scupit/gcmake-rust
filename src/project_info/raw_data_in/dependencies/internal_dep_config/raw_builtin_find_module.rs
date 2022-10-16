use serde::{Deserialize};

use super::{ComponentsFindModuleLinks, raw_target_config_common::RawPredefinedTargetMapIn, RawMutualExclusionSet, raw_dep_common::{RawPredepCommon, RawEmscriptenConfig}};

#[derive(Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct BuiltinFindModuleNamespaceConfig {
  pub cmakelists_linking: String
}

#[derive(Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub enum CMakeModuleType {
  ConfigFile,
  BuiltinFindModule,
  CustomFindModule
}

#[derive(Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct RawModuleDep {
  pub found_var: String,
  pub module_type: CMakeModuleType,
  pub links: ComponentsFindModuleLinks,
  pub namespace_config: BuiltinFindModuleNamespaceConfig,
  pub mutually_exclusive: Option<RawMutualExclusionSet>,
  pub emscripten_config: Option<RawEmscriptenConfig>,
  pub targets: RawPredefinedTargetMapIn
}

impl RawPredepCommon for RawModuleDep {
  fn can_trivially_cross_compile(&self) -> bool {
    false
  }

  fn maybe_mutual_exclusion_groups(&self) -> &Option<RawMutualExclusionSet> {
    &self.mutually_exclusive
  }

  fn raw_target_map_in(&self) -> &RawPredefinedTargetMapIn {
    &self.targets
  }

  fn repo_url(&self) -> Option<&str> {
    None
  }

  fn github_url(&self) -> Option<&str> {
    None
  }

  fn get_emscripten_config(&self) -> Option<&RawEmscriptenConfig> {
    self.emscripten_config.as_ref()
  }

  fn supports_emscripten(&self) -> bool {
    self.emscripten_config.is_some()
  }

  fn is_internally_supported_by_emscripten(&self) -> bool {
    return match &self.emscripten_config {
      None => false,
      Some(config) => match (&config.is_internally_supported, &config.link_flag) {
        (Some(true), _) => true,
        (_, Some(_)) => true,
        _ => false
      }
    }
  }

  fn supports_git_download_method(&self) -> bool {
    false
  }

  fn supports_url_download_method(&self) -> bool {
    false
  }
}
