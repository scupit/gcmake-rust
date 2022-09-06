mod raw_components_builtin_find_module;
mod raw_predefined_subdirectory_dep;
mod raw_builtin_find_module;
mod raw_target_config_common;

pub mod raw_dep_common;

pub use raw_predefined_subdirectory_dep::*;
pub use raw_components_builtin_find_module::*;
pub use raw_builtin_find_module::*;
pub use raw_target_config_common::*;

use std::{collections::HashMap, path::{PathBuf, Path}, fs, io, rc::Rc};
use serde::{Deserialize};

use self::raw_dep_common::RawPredepCommon;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SingleRawPredefinedDependencyConfigGroup {
  pub as_subdirectory: Option<RawSubdirectoryDependency>,
  pub cmake_components_module: Option<RawComponentsModuleDep>,
  pub cmake_module: Option<RawModuleDep>
}

impl SingleRawPredefinedDependencyConfigGroup {
  pub fn get_common(&self) -> Result<&dyn RawPredepCommon, String> {
    let first_available: Option<&dyn RawPredepCommon> = if let Some(subdir_dep) = &self.as_subdirectory {
      Some(subdir_dep)
    }
    else if let Some(components_dep) = &self.cmake_components_module {
      Some(components_dep)
    }
    else if let Some(module_dep) = &self.cmake_module {
      Some(module_dep)
    }
    else {
      None
    };
    
    return match first_available {
      Some(available_config) => Ok(available_config),
      None => Err(format!("The dependency doesn't contain a configuration."))
    }
  }
}

pub struct PredefinedCMakeDepHookFile {
  pub file_path: PathBuf,
  contents: String
}

impl PredefinedCMakeDepHookFile {
  pub fn new(file_path: impl AsRef<Path>) -> io::Result<Option<Self>> {
    return if file_path.as_ref().is_file() {
      // println!("EXISTS: {}", file_path.as_ref().to_str().unwrap());
      Ok(Some(Self {
        file_path: file_path.as_ref().to_path_buf(),
        contents: fs::read_to_string(file_path.as_ref())?
      }))
    }
    else {
      // println!("doesn't exist: {}", file_path.as_ref().to_str().unwrap());
      Ok(None)
    }
  }

  pub fn contents_ref(&self) -> &str {
    &self.contents
  }
}

pub struct RawPredefinedDependencyInfo {
  pub dep_configs: SingleRawPredefinedDependencyConfigGroup,
  pub pre_load: Option<Rc<PredefinedCMakeDepHookFile>>,
  pub post_load: Option<Rc<PredefinedCMakeDepHookFile>>,
  pub custom_populate: Option<Rc<PredefinedCMakeDepHookFile>>
}

// Container for all dependency types defined in supported_dependencies.yaml
pub type AllRawPredefinedDependencies = HashMap<String, RawPredefinedDependencyInfo>;
