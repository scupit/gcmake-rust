pub mod internal_dep_config;
pub mod user_given_dep_config;

use std::{path::{PathBuf, Path}, fs::{DirEntry, self}, rc::Rc};

use crate::{program_actions::gcmake_dep_config_dir};

use self::internal_dep_config::{AllRawPredefinedDependencies, SingleRawPredefinedDependencyConfigGroup, RawPredefinedDependencyInfo, PredefinedCMakeDepHookFile};
use colored::*;

fn load_hook_file(
  entry_path: impl AsRef<Path>,
  dep_dir_name: &str,
  file_name: impl AsRef<str>
) -> Result<Option<Rc<PredefinedCMakeDepHookFile>>, String> {
  let file_name_str: &str = file_name.as_ref();

  return Ok(
    PredefinedCMakeDepHookFile::new(entry_path.as_ref().join(file_name_str))
      .map_err(|err| format!(
        "{} loading {} hook file for predefined dependency {}: {}",
        "Error".red(),
        file_name_str,
        dep_dir_name,
        err.to_string()
      ))?
      .map(|hook_file| Rc::new(hook_file))
  );
}

pub fn all_raw_supported_dependency_configs() -> Result<AllRawPredefinedDependencies, String> {
  /*
    Whole bunch of TODOS related to the new dependency configuration system.
    ================================================================================

    configuration repository should be located in ~/.gcmake/gcmake-dependency-configs
    WHERE ~ is HOME env var on Unix and USERPROFILE on Windows.

    TODOS:
      1 (DONE). Get all configurations from the external dependency repository if the repository is present.
      2. Otherwise, prompt the user asking to "download the dependency configuration repository to
          ~/.gcmake-/gcmake-dependency-configs" using the same steps as TODO #3.
      3 (DONE). Add an 'dep-config update [--branch <branch>]' command to clone the repo if it doesn't exist,
        checkout the given branch, and pull the latest changes on that branch.
      4. Generate a single file yaml index of all dependency configurations post-pull success.
          I'm not sure if this is needed or not, so this is more of a quality-of-life convenience step.
  */

  let dep_config_repo: PathBuf = gcmake_dep_config_dir();

  if !dep_config_repo.is_dir() {
    return Err(format!(
      "Failed to retrieve dependency information because the 'external dependency configuration repository' was not found on the local system (should be at {}). Running `gcmake dep-config update` should fix the issue.",
      dep_config_repo.to_str().unwrap()
    ));
  }

  let mut all_dep_configs: AllRawPredefinedDependencies = AllRawPredefinedDependencies::new();

  // TODO: Refactor this. Currently, all dependency configs (including all their cmake scripts) are
  // loaded every run of gcmake. They should only be loaded as needed.
  let dir_data = fs::read_dir(&dep_config_repo)
    .map_err(|err| err.to_string())?;

  for maybe_entry in dir_data {
    let entry: DirEntry = maybe_entry.map_err(|err| err.to_string())?;
    let entry_path: PathBuf = entry.path();
    let dep_dir_name: &str = entry_path.file_name().unwrap().to_str().unwrap();

    if entry_path.is_dir() && !dep_dir_name.starts_with('.') {
      // let dep_name = entry_path.file_name().unwrap();
      let mut config_file_path: PathBuf = entry.path();
      config_file_path.push("dep_config.yaml");

      let config_file_contents: String = fs::read_to_string(&config_file_path)
        .map_err(|err| err.to_string())?;

      let dep_configs: SingleRawPredefinedDependencyConfigGroup = serde_yaml::from_str(&config_file_contents)
        .map_err(|err| format!(
          "{} loading dependency config info for predefined dependency {}:\n\t{}",
          "Error".red(),
          dep_dir_name.green(),
          err.to_string()
        ))?;

      let find_module_base_name: &str = dep_configs.get_common()?.find_module_base_name().unwrap_or(dep_dir_name);
      
      let dep_config_container = RawPredefinedDependencyInfo {
        custom_find_module: load_hook_file(
          &entry_path,
          dep_dir_name,
          format!("Find{}.cmake", find_module_base_name)
        )?,
        dep_configs,
        pre_load: load_hook_file(&entry_path, dep_dir_name, "pre_load.cmake")?,
        post_load: load_hook_file(&entry_path, dep_dir_name, "post_load.cmake")?,
        custom_populate: load_hook_file(&entry_path, dep_dir_name, "custom_populate.cmake")?,
      };

      if let Some(subdir_dep) = &dep_config_container.dep_configs.as_subdirectory {
        if subdir_dep.requires_custom_fetchcontent_populate && dep_config_container.custom_populate.is_none() {
          return Err(format!(
            "Predefined dependency '{}' as_subdirectory configuration requires a custom_populate.cmake. However, one could not be found in the '{}' configuration directory.",
            dep_dir_name,
            dep_dir_name
          ))
        }
      }

      all_dep_configs.insert(
        dep_dir_name.to_string(),
        dep_config_container
      );
    }
  }

  return Ok(all_dep_configs);
}