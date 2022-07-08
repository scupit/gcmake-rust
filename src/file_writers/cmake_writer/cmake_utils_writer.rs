use std::{collections::HashMap, fs::{self}, io, iter::FromIterator, path::{PathBuf}};

use super::ordered_utils;

pub struct CMakeUtilFile {
  pub util_name: &'static str,
  pub util_contents: &'static str
}

pub struct CMakeUtilWriter {
  cmake_utils_path: PathBuf,
  utils: Vec<CMakeUtilFile>
}

impl CMakeUtilWriter {
  pub fn new(cmake_utils_path: PathBuf) -> Self {
    return Self {
      cmake_utils_path,
      // TODO: Make all these their own *.cmake files, so they are easier to maintain.
      // Load them here using a pre-build script.
      utils: ordered_utils::ordered_utils_vec()
    }
  }

  pub fn write_cmake_utils(&self) -> io::Result<()> {
    if !self.cmake_utils_path.is_dir() {
      fs::create_dir(&self.cmake_utils_path)?;
    }

    // for (util_name, util_contents) in &self.utils {
    for CMakeUtilFile {util_name, util_contents} in &self.utils {
      let mut util_file_path = self.cmake_utils_path.join(util_name);
      util_file_path.set_extension("cmake");

      fs::write(
        util_file_path,
        util_contents
      )?;
    }

    Ok(())
  }

  pub fn get_utils(&self) -> &Vec<CMakeUtilFile> {
    &self.utils
  }
}