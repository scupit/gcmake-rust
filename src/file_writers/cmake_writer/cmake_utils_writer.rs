use std::{fs::{self}, io, path::{PathBuf, Path}};

use super::ordered_utils;

const CMAKE_FIND_MODULES_DIRNAME: &'static str = "modules";

pub struct CMakeUtilFile {
  pub util_name: &'static str,
  pub util_contents: &'static str
}

impl CMakeUtilFile {
  pub fn is_find_module(&self) -> bool {
    self.util_name.starts_with("Find")
  }
}

pub struct CMakeUtilWriter {
  cmake_utils_path: PathBuf,
  custom_find_modules_path: PathBuf,
  utils: Vec<CMakeUtilFile>
}

impl CMakeUtilWriter {
  pub fn new(cmake_utils_path: PathBuf) -> Self {
    return Self {
      custom_find_modules_path: cmake_utils_path.join(CMAKE_FIND_MODULES_DIRNAME),
      cmake_utils_path,
      utils: ordered_utils::ordered_utils_vec()
    }
  }

  pub fn write_cmake_utils(&self) -> io::Result<()> {
    if !self.cmake_utils_path.is_dir() {
      fs::create_dir(&self.cmake_utils_path)?;
    }

    if !self.custom_find_modules_path.is_dir() {
      fs::create_dir(&self.custom_find_modules_path)?;
    }

    for util_config in &self.utils {
      let mut util_file_path: PathBuf = self.cmake_utils_path.clone();

      if util_config.is_find_module() {
        util_file_path.push(CMAKE_FIND_MODULES_DIRNAME);
      }

      util_file_path.push(util_config.util_name);
      util_file_path.set_extension("cmake");

      fs::write(
        util_file_path,
        util_config.util_contents
      )?;
    }

    Ok(())
  }

  pub fn copy_custom_find_file(&self, file_path: impl AsRef<Path>) -> io::Result<()> {
    let file_name: &str = file_path.as_ref().file_name().unwrap().to_str().unwrap();

    fs::copy(
      file_path.as_ref(),
      self.custom_find_modules_path.join(file_name)
    )?;

    Ok(())
  }

  pub fn get_utils(&self) -> &Vec<CMakeUtilFile> {
    &self.utils
  }
}