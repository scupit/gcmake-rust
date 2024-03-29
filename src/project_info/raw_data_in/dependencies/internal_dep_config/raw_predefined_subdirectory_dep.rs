use std::collections::HashMap;

use serde::Deserialize;
use super::{raw_target_config_common::RawPredefinedTargetMapIn, RawMutualExclusionSet, raw_dep_common::{RawPredepCommon, RawEmscriptenConfig, RawDebianPackagesConfig, RawDepConfigOption}};

#[derive(Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct RawSubdirDepLinks {
  pub github: Option<String>,
  pub gcmake_readme: Option<String>
}

#[derive(Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct NamespaceConfig {
  pub cmakelists_linking: String
}

#[derive(Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct RawSubdirDepGitRepoConfig {
  pub repo_url: String
}

#[derive(Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct RawExtensionsByPlatform {
  pub windows: String,
  pub unix: String
}

#[derive(Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct RawSubdirDepUrlDownloadConfig {
  pub url_base_by_version: Option<HashMap<String, String>>,
  pub url_base: Option<String>,
  pub version_transform: String,
  pub extensions: RawExtensionsByPlatform
}

#[derive(Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct RawDownloadInfo {
  pub git_method: Option<RawSubdirDepGitRepoConfig>,
  pub url_method: Option<RawSubdirDepUrlDownloadConfig>
}

fn default_requires_custom_populate() -> bool { false }

// A predefined dependency which exists within the project build tree.
// These should always be inside the dep/ folder.
#[derive(Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct RawSubdirectoryDependency {
  pub namespace_config: NamespaceConfig,
  // Name of the include directory the library will be installed to. Usually this is the same as the
  // project name, but not always. For example, nlohmann_json installs its package config to the
  // nlohmann_json directory, but uses 'nlohmann' as its include dir.
  pub installed_include_dir_name: Option<String>,
  // Name of the directory the project is installed to. The directory name should be the name
  // of the project the 
  pub config_file_project_name: Option<String>,
  pub links: Option<RawSubdirDepLinks>,
  pub download_info: RawDownloadInfo,
  pub target_configs: RawPredefinedTargetMapIn,
  pub mutually_exclusive: Option<RawMutualExclusionSet>,
  pub emscripten_config: Option<RawEmscriptenConfig>,
  pub debian_packages: Option<RawDebianPackagesConfig>,
  pub config_options: Option<HashMap<String, RawDepConfigOption>>,

  #[serde(default = "default_requires_custom_populate")]
  pub requires_custom_fetchcontent_populate: bool,

  // CMake variable used for installation
  pub install_var: Option<String>,
  pub inverse_install_var: Option<String>,
  pub install_by_default: Option<bool>,
  
  #[serde(rename = "can_cross_compile")]
  _can_cross_compile: bool
}

impl RawSubdirectoryDependency {
  pub fn get_url_info(&self) -> Option<&RawSubdirDepUrlDownloadConfig> {
    self.download_info.url_method.as_ref()
  }
}

impl RawPredepCommon for RawSubdirectoryDependency {
  fn find_module_base_name(&self) -> Option<&str> {
    None
  }

  fn raw_debian_packages_config(&self) -> Option<&RawDebianPackagesConfig> {
    self.debian_packages.as_ref()
  }

  fn can_trivially_cross_compile(&self) -> bool {
    self._can_cross_compile
  }

  fn maybe_mutual_exclusion_groups(&self) -> &Option<RawMutualExclusionSet> {
    &self.mutually_exclusive
  }

  fn raw_target_map_in(&self) -> &RawPredefinedTargetMapIn {
    &self.target_configs
  }

  fn repo_url(&self) -> Option<&str> {
    return match &self.download_info.git_method {
      Some(RawSubdirDepGitRepoConfig { repo_url }) => Some(repo_url),
      _ => None
    }
  }

  fn github_url(&self) -> Option<&str> {
    match &self.links {
      None => None,
      Some(links) => links.github.as_ref().map(|github_link| &github_link[..])
    }
  }

  fn gcmake_readme_url(&self) -> Option<&str> {
    match &self.links {
      None => None,
      Some(links) => links.gcmake_readme.as_ref().map(|gcmake_readme_link| &gcmake_readme_link[..])
    }
  }

  fn get_emscripten_config(&self) -> Option<&RawEmscriptenConfig> {
    self.emscripten_config.as_ref()
  }

  fn supports_emscripten(&self) -> bool {
    self.can_trivially_cross_compile() || self.emscripten_config.is_some()
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
    return self.download_info.git_method.is_some();
  }

  fn supports_url_download_method(&self) -> bool {
    return self.download_info.url_method.is_some();
  }

  fn config_options_map(&self) -> Option<&HashMap<String, RawDepConfigOption>> {
    self.config_options.as_ref()
  }
}
