use std::{collections::{HashMap, HashSet, BTreeMap}};

use colored::Colorize;

use crate::project_info::{raw_data_in::dependencies::{internal_dep_config::{RawSubdirectoryDependency, raw_dep_common::{RawPredepCommon, RawEmscriptenConfig}, RawExtensionsByPlatform, RawSubdirDepUrlDownloadConfig}, user_given_dep_config::{UserGivenPredefinedDependencyConfig}}, parsers::{version_parser::{parse_version, ThreePartVersion}, version_transform_parser::transform_version}, path_manipulation::without_leading_dot};

use super::{predep_module_common::{PredefinedDepFunctionality, FinalDebianPackagesConfig, FinalDepConfigOption, resolve_final_config_options}, final_target_map_common::{FinalTargetConfigMap, make_final_target_config_map}};

#[derive(Clone)]
pub enum GitRevisionSpecifier {
  Tag(String),
  CommitHash(String)
}

#[derive(Clone)]
pub struct FinalGitRepoDescriptor {
  pub repo_url: String,
  pub revision_specifier: GitRevisionSpecifier
}

#[derive(Clone)]
pub struct FinalUrlExtensions {
  windows: String,
  unix: String
}

#[derive(Clone)]
pub struct FinalUrlDownloadDescriptor {
  pub url_without_extension: String,
  pub extension: FinalUrlExtensions
}

impl FinalUrlDownloadDescriptor {
  pub fn windows_url(&self) -> String {
    return format!("{}.{}", &self.url_without_extension, &self.extension.windows);
  }

  pub fn unix_url(&self) -> String {
    return format!("{}.{}", &self.url_without_extension, &self.extension.unix);
  }
}

#[derive(Clone)]
pub enum FinalDownloadMethod {
  UrlMode(FinalUrlDownloadDescriptor),
  GitMode(FinalGitRepoDescriptor)
}

impl FinalDownloadMethod {
  pub fn git_details(&self) -> Option<&FinalGitRepoDescriptor> {
    return match self {
      Self::GitMode(git_config) => Some(git_config),
      _ => None
    }
  }

  pub fn is_git(&self) -> bool {
    return match self {
      Self::GitMode(_) => true,
      _ => false
    }
  }
}

#[derive(Clone)]
pub struct SubdirDepInstallationConfig {
  pub var_name: String,
  pub is_inverse: bool,
  pub should_install_by_default: bool
}

impl SubdirDepInstallationConfig {
  pub fn actual_value_for(&self, should_install: bool) -> bool {
    return if self.is_inverse
      { !should_install }
      else { should_install }
  }
}

fn get_url_from_version(
  version: &ThreePartVersion,
  url_info: &RawSubdirDepUrlDownloadConfig
) -> Result<String, String> {
  if let Some(url_map) = url_info.url_base_by_version.as_ref() {
    let mut version_list: Vec<(ThreePartVersion, &str)> = Vec::new();

    for (version_str_key, matching_url) in url_map {
      match ThreePartVersion::from_str(version_str_key) {
        None => return Err(format!("Invalid version key \"{}\". The key is for base URL: \"{}\"", version_str_key.red(), matching_url)),
        Some(parsed_version) => {
          version_list.push((parsed_version, matching_url.as_str()));
        }
      }
    }

    version_list.sort_by(|(first_version, _), (second_version, _)|
      first_version.cmp(second_version).reverse()
    );

    if let Some((_, matching_url)) = version_list.into_iter().find(|(parsed_version, _)| version >= parsed_version) {
      return Ok(matching_url.to_string());
    }
  }

  if let Some(base_url) = url_info.url_base.as_ref() {
    return Ok(base_url.clone());
  }

  return Err(format!("No matching URL found for version '{}'", version.to_string().red()))
}

fn resolve_download_method(
  subdir_dep: &RawSubdirectoryDependency,
  user_given_config: &UserGivenPredefinedDependencyConfig,
  dep_name: &str
) -> Result<FinalDownloadMethod, String> {
  if !(user_given_config.specifies_git_mode_options() || user_given_config.specifies_url_mode_options()) {
    let supported_modes: Vec<&str> = [
      ("Git", subdir_dep.supports_git_download_method()),
      ("URL", subdir_dep.supports_url_download_method()),
    ]
      .iter()
      .filter_map(|(method_name, is_supported)| {
        if *is_supported { Some(*method_name) }
        else { None }
      })
      .collect();

    let mode_string: String = format!(
      "{}",
      supported_modes.join(" or ")
    );

    return Err(format!(
      "Predefined dependency configuration for '{}' doesn't specify any download configuration options. Please specify {} mode configuration options.",
      dep_name,
      mode_string
    ));
  }

  match (subdir_dep.supports_git_download_method(), subdir_dep.supports_url_download_method()) {
    (false, false) => {
      return Err(format!(
        "Predefined dependency '{}' supports no download methods. This is a configuration error, and shouldn't happen. If you see this, try updating your local copy of the dependency configuration repository to the latest version by running {}",
        dep_name,
        "gcmake-rust dep-config update -b develop".magenta()
      ));
    }
    (true, false) => {
      if !user_given_config.specifies_git_mode_options() {
        return Err(format!(
          "Predefined dependency '{}' only supports the git download method, but doesn't specify git download options. Make sure to specify a revision, such as git_tag: v1.1.0",
          dep_name
        ));
      }

      if user_given_config.specifies_url_mode_options() {
        return Err(format!(
          "Predefined dependency '{}' only supports the git download method, but specifies URL mode options. URL options such as 'file_version' shouldn't be specified.",
          dep_name
        ));
      }
    },
    (false, true) => {
      if !user_given_config.specifies_url_mode_options() {
        return Err(format!(
          "Predefined dependency '{}' only supports the URL download method, but doesn't specify URL download options. Make sure to specify an archive version, such as file_version: v1.1.0",
          dep_name
        ));
      }

      if user_given_config.specifies_git_mode_options() {
        return Err(format!(
          "Predefined dependency '{}' only supports the URL download method, but specifies some git mode options. Git options such as 'git_tag' shouldn't be specified.",
          dep_name
        ));
      }
    },
    (true, true) => {
      // TODO: Make this not an error. If git isn't found on the system, fall back to URL download mode if possible.
      if user_given_config.specifies_git_mode_options() && user_given_config.specifies_url_mode_options() {
        return Err(format!(
          "Predefined dependency configuration for '{}' specifies options for both Git and URL download modes. Only options for one of the modes should be specified.",
          dep_name
        ));
      }
    }
  }

  // As of this point, any specified configuration options are valid. We should use the mode determined by
  // thos configuration options.

  if user_given_config.specifies_git_mode_options() {
    let revision_specifier: GitRevisionSpecifier = if let Some(tag_string) = &user_given_config.git_tag {
      GitRevisionSpecifier::Tag(tag_string.clone())
    }
    else if let Some(hash_string) = &user_given_config.commit_hash {
      GitRevisionSpecifier::CommitHash(hash_string.clone())
    }
    else {
      return Err(format!("Must specify either a commit_hash or git_tag for dependency '{}'", dep_name));
    };

    return Ok(FinalDownloadMethod::GitMode(FinalGitRepoDescriptor {
      revision_specifier,
      repo_url: match &user_given_config.repo_url {
        None => subdir_dep.download_info.git_method.as_ref().unwrap().repo_url.clone(),
        Some(alternate_url) => alternate_url.clone()
      }
    }))
  }
  else {
    let given_version: &str = user_given_config.file_version.as_ref().unwrap();

    match parse_version(given_version) {
      None => return Err(format!(
        "Invalid file_version '{}' given to predefined dependency '{}' configuration",
        given_version,
        dep_name
      )),
      Some(parsed_version) => {
        let transformation_str: &str = &subdir_dep.download_info.url_method.as_ref().unwrap().version_transform;
        match transform_version(&parsed_version, transformation_str) {
          Ok(transformed_version) => {
            let RawExtensionsByPlatform {
              windows: windows_url_extension,
              unix: unix_url_extension
            } = &subdir_dep.get_url_info().unwrap().extensions;

            let base_url: String = get_url_from_version(&parsed_version, subdir_dep.get_url_info().unwrap())
              .map_err(|err_msg|
                format!(
                  "When resolving base URL for dependency '{}':\n{}",
                  dep_name.yellow(),
                  err_msg
                )
              )?;

            return Ok(FinalDownloadMethod::UrlMode(FinalUrlDownloadDescriptor {
              // <baseUrl><transformedVersion>
              url_without_extension: format!(
                "{}{}",
                base_url,
                transformed_version
              ),
              extension: FinalUrlExtensions {
                windows: without_leading_dot(windows_url_extension),
                unix: without_leading_dot(unix_url_extension)
              }
            }));
          },
          Err(err_msg) => return Err(format!(
            "Failed to transform version to valid URL for predefined dependency '{}' configuration. Details:\n{}",
            dep_name,
            err_msg
          ))
        }
      }
    }
  }

}

#[derive(Clone)]
pub struct PredefinedSubdirDep {
  // git_repo: FinalGitRepoDescriptor,
  _download_method: FinalDownloadMethod,
  installed_include_dir_name: Option<String>,
  // Unused for now, but may be used in the future to propagate installed DLLs from the gcmake project
  // install dir on Windows. I'm not sure if that's a good idea or not.
  _config_file_project_name: Option<String>,
  debian_packages: FinalDebianPackagesConfig,
  // Map of target base name to the namespaced target name used for linking.
  target_map: FinalTargetConfigMap,
  cmake_namespaced_target_map: HashMap<String, String>,
  yaml_namespaced_target_map: HashMap<String, String>,
  requires_custom_populate: bool,
  installation_details: Option<SubdirDepInstallationConfig>,
  raw_dep: RawSubdirectoryDependency,
  config_options: BTreeMap<String, FinalDepConfigOption>
}

impl PredefinedSubdirDep {
  pub fn get_target_config_map(&self) -> &FinalTargetConfigMap {
    &self.target_map
  }

  pub fn get_installation_details(&self) -> &Option<SubdirDepInstallationConfig> {
    &self.installation_details
  }

  pub fn custom_relative_include_dir_name(&self) -> &Option<String> {
    &self.installed_include_dir_name
  }

  pub fn requires_custom_fetchcontent_populate(&self) -> bool {
    self.requires_custom_populate
  }

  pub fn get_cmake_linkable_target_name(&self, target_name: &str) -> Option<&str> {
    self.cmake_namespaced_target_map.get(target_name)
      .map(|str_ref| &str_ref[..])
  }

  pub fn get_yaml_linkable_target_name(&self, target_name: &str) -> Option<&str> {
    self.yaml_namespaced_target_map.get(target_name)
      .map(|str_ref| &str_ref[..])
  }

  pub fn download_method(&self) -> &FinalDownloadMethod {
    &self._download_method
  }

  pub fn from_subdir_dep(
    subdir_dep: &RawSubdirectoryDependency,
    user_given_config: &UserGivenPredefinedDependencyConfig,
    dep_name: &str,
    valid_feature_list: Option<&Vec<&str>>
  ) -> Result<Self, String> {
    
    let target_map = make_final_target_config_map(dep_name, subdir_dep, valid_feature_list)
      .map_err(|err_msg| format!(
        "When loading predefined subdirectory dependency \"{}\":\n\n{}",
        dep_name,
        err_msg
      ))?;

    let mut cmake_namespaced_target_map: HashMap<String, String> = HashMap::new();

    for (target_name, target_config) in &target_map {
      cmake_namespaced_target_map.insert(
        target_name.to_string(),
        format!(
          "{}{}",
          &subdir_dep.namespace_config.cmakelists_linking,
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
          dep_name.to_string(),
          &target_config.cmake_yaml_name
        )
      );
    }

    let should_install_by_default: bool = subdir_dep.install_by_default.unwrap_or(true);

    let installation_details: Option<SubdirDepInstallationConfig> = match (&subdir_dep.install_var, &subdir_dep.inverse_install_var) {
      (Some(install_var), _) => {
        Some(SubdirDepInstallationConfig {
          var_name: install_var.clone(),
          is_inverse: false,
          should_install_by_default
        })
      },
      (_, Some(inverse_install_var)) => {
        Some(SubdirDepInstallationConfig {
          var_name: inverse_install_var.clone(),
          is_inverse: true,
          should_install_by_default
        })
      },
      _ => None
    };

    return Ok(
      Self {
        _download_method: resolve_download_method(subdir_dep, user_given_config, dep_name)?,
        installed_include_dir_name: subdir_dep.installed_include_dir_name.clone(),
        _config_file_project_name: subdir_dep.config_file_project_name.clone(),
        target_map,
        debian_packages: FinalDebianPackagesConfig::make_from(subdir_dep.raw_debian_packages_config()),
        cmake_namespaced_target_map,
        yaml_namespaced_target_map,
        requires_custom_populate: subdir_dep.requires_custom_fetchcontent_populate,
        installation_details,
        raw_dep: subdir_dep.clone(),
        config_options: resolve_final_config_options(
          subdir_dep.config_options_map(),
          user_given_config.options.clone()
        )
          .map_err(|err_msg| format!(
            "In configuration for predefined dependency '{}':\n{}",
            dep_name.yellow(),
            err_msg
          ))?
      }
    )
  }
}

impl PredefinedDepFunctionality for PredefinedSubdirDep {
  fn debian_packages_config(&self) -> &FinalDebianPackagesConfig {
    &self.debian_packages
  }

  fn can_cross_compile(&self) -> bool {
    self.raw_dep.can_trivially_cross_compile()
  }

  fn get_target_config_map(&self) -> &FinalTargetConfigMap {
    &self.target_map
  }

  fn target_name_set(&self) -> HashSet<String> {
    self.target_map.keys()
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