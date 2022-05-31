pub mod internal_dep_config;
pub mod user_given_dep_config;

use std::{path::PathBuf, fs::{DirEntry, self}};

use crate::program_actions::local_dep_config_repo_location;

use self::internal_dep_config::{AllRawPredefinedDependencies, SingleRawPredefinedDependencyInfo};

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

  let dep_config_repo: PathBuf = local_dep_config_repo_location();

  if !dep_config_repo.is_dir() {
    return Err(format!(
      "Failed to retrieve dependency information because the 'external dependency configuration repository' was not found on the local system (should be at {}). Running `gcmake dep-config update` should fix the issue.",
      dep_config_repo.to_str().unwrap()
    ));
  }

  let mut all_dep_configs: AllRawPredefinedDependencies = AllRawPredefinedDependencies::new();

  let dir_data = fs::read_dir(&dep_config_repo)
    .map_err(|err| err.to_string())?;

  for maybe_entry in dir_data {
    let entry: DirEntry = maybe_entry.map_err(|err| err.to_string())?;
    let entry_path: PathBuf = entry.path();
    let file_name: &str = entry_path.file_name().unwrap().to_str().unwrap();

    if entry_path.is_dir() && !file_name.starts_with('.') {
      // let dep_name = entry_path.file_name().unwrap();
      let mut config_file_path: PathBuf = entry.path();
      config_file_path.push("dep_config.yaml");

      let config_file_contents: String = fs::read_to_string(&config_file_path)
        .map_err(|err| err.to_string())?;

      let single_dep_config: SingleRawPredefinedDependencyInfo = serde_yaml::from_str(&config_file_contents)
        .map_err(|err| err.to_string())?;

      all_dep_configs.insert(
        file_name.to_string(),
        single_dep_config
      );
    }
  }

  return Ok(all_dep_configs);
}