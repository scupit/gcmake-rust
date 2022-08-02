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

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SingleRawPredefinedDependencyConfigGroup {
  pub as_subdirectory: Option<RawSubdirectoryDependency>,
  pub cmake_components_module: Option<RawComponentsModuleDep>,
  pub cmake_module: Option<RawModuleDep>
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
