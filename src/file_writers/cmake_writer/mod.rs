mod cmake_utils_writer;
mod cmakelists_writer;

use std::{io};

use crate::project_info::final_project_data::UseableFinalProjectDataGroup;

use self::cmakelists_writer::configure_cmake_helper;

pub fn configure_cmake(project_data: &UseableFinalProjectDataGroup) -> io::Result<()> {
  configure_cmake_helper(&project_data.root_project)
}
