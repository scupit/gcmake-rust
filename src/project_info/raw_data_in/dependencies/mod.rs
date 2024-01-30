pub mod internal_dep_config;
pub mod user_given_dep_config;

use std::{cell::RefCell, collections::{HashMap, HashSet}, fs::{DirEntry, self}, path::{PathBuf, Path}, rc::Rc};

use crate::program_actions::gcmake_dep_config_dir;

use self::internal_dep_config::{SingleRawPredefinedDependencyConfigGroup, RawPredefinedDependencyInfo, PredefinedCMakeDepHookFile};
use colored::*;

type InternalDepHashMap = HashMap<String, RawPredefinedDependencyInfo>;

// This is a lazy "HashMap" of predefined dependency configurations.
// Since loading dependency configurations can be expensive (requires multiple file reads per dependency),
// configs are only loaded when first requested.
pub struct RawPredefinedDependencyMap {
  dep_config_dir: PathBuf,
  allowed_config_names: HashSet<String>,
  configs: RefCell<InternalDepHashMap>
}

impl RawPredefinedDependencyMap {
  unsafe fn get_configs(&self) -> *const InternalDepHashMap {
    self.configs.as_ptr()
  }

  pub fn available_dep_names(&self) -> &HashSet<String> {
    &self.allowed_config_names
  }

  pub fn get(&self, config_name: &str) -> Result<Option<&RawPredefinedDependencyInfo>, String> {
    unsafe {
      if let Some(existing_config) = (*self.get_configs()).get(config_name) {
        return Ok(Some(existing_config));
      }
    }

    if !self.allowed_config_names.contains(config_name) {
      return Ok(None);
      // return Err(format!("Tried to retrieve configuration for dependency '{}', which doesn't exist. Did you misspell the dependency name?", config_name.yellow()))
    }

    let mut config_file_path: PathBuf = self.dep_config_dir.join(config_name);
    config_file_path.push("dep_config.yaml");

    let config_file_contents: String = fs::read_to_string(&config_file_path)
      .map_err(|err| err.to_string())?;

    let dep_configs: SingleRawPredefinedDependencyConfigGroup = serde_yaml::from_str(&config_file_contents)
      .map_err(|err| format!(
        "{} loading dependency config info for predefined dependency {}:\n\t{}",
        "Error".red(),
        config_name.green(),
        err.to_string()
      ))?;

    let find_module_base_name: &str = dep_configs.get_common()?.find_module_base_name().unwrap_or(config_name);
    
    let config_dir: &Path = self.dep_config_dir.as_path();
    let dep_config_container = RawPredefinedDependencyInfo {
      custom_find_module: load_hook_file(
        config_dir,
        config_name,
        &format!("Find{}.cmake", find_module_base_name)
      )?,
      dep_configs,
      pre_load: load_hook_file(config_dir, config_name, "pre_load.cmake")?,
      post_load: load_hook_file(config_dir, config_name, "post_load.cmake")?,
      custom_populate: load_hook_file(config_dir, config_name, "custom_populate.cmake")?,
    };

    if let Some(subdir_dep) = &dep_config_container.dep_configs.as_subdirectory {
      if subdir_dep.requires_custom_fetchcontent_populate && dep_config_container.custom_populate.is_none() {
        return Err(format!(
          "Predefined dependency '{}' as_subdirectory configuration requires a '{}'. However, one could not be found in the '{}' configuration directory.",
          config_name.yellow(),
          "custom_populate.cmake".yellow(),
          config_name.yellow()
        ));
      }
    }

    self.configs.borrow_mut().insert(config_name.to_string(), dep_config_container);
    return Ok(Some(self.unchecked_get(config_name)));
  }

  pub fn unchecked_get(&self, config_name: &str) -> &RawPredefinedDependencyInfo  {
    unsafe {
      return (*self.get_configs()).get(config_name).unwrap();
    }
  }

  pub fn new(dep_config_repo_dir: &Path) -> Result<Self, String> {
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
    */

    if !dep_config_repo_dir.is_dir() {
      return Err(format!(
        "Failed to retrieve dependency information because the 'external dependency configuration repository' was not found on the local system (should be at {}). Running `gcmake dep-config update` should fix the issue.",
        dep_config_repo_dir.to_str().unwrap()
      ));
    }

    let mut allowed_config_names: HashSet<String> = HashSet::new();

    let dir_data = fs::read_dir(&dep_config_repo_dir)
      .map_err(|err| err.to_string())?;

    for maybe_entry in dir_data {
      let entry: DirEntry = maybe_entry.map_err(|err| err.to_string())?;
      let entry_path: PathBuf = entry.path();
      let dep_dir_name: &str = entry_path.file_name().unwrap().to_str().unwrap();

      if entry_path.is_dir() && !dep_dir_name.starts_with('.') {
        allowed_config_names.insert(dep_dir_name.to_string());
      }
    }

    return Ok(Self {
      allowed_config_names,
      dep_config_dir: dep_config_repo_dir.to_path_buf(),
      configs: RefCell::new(HashMap::new())
    });
  }
}

fn load_hook_file(
  config_root_dir: &Path,
  dep_dir_name: &str,
  file_name: &str
) -> Result<Option<Rc<PredefinedCMakeDepHookFile>>, String> {
  let file_name_str: &str = file_name.as_ref();
  let file_path: PathBuf = config_root_dir.join(dep_dir_name).join(file_name_str);

  return Ok(
    PredefinedCMakeDepHookFile::new(file_path.as_path())
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

pub fn all_raw_supported_dependency_configs() -> Result<RawPredefinedDependencyMap, String> {
  return RawPredefinedDependencyMap::new(gcmake_dep_config_dir().as_path());
}