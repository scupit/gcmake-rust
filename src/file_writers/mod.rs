use std::{io};
use crate::project_info::final_project_data::{FinalProjectData, UseableFinalProjectDataGroup};

use self::{cmakelists_writer::configure_cmake};

mod cmake_utils_writer;
mod cmakelists_writer;

pub struct ProjectWriteConfiguration {
  name: String,
  config_func: fn(&UseableFinalProjectDataGroup) -> io::Result<()>,
}

pub fn write_configurations<'a, FBefore, FAfter>(
  project_data: &UseableFinalProjectDataGroup,
  before_write: FBefore,
  after_write: FAfter
)
  where
    FBefore: Fn(&str),
    FAfter: Fn((&str, io::Result<()>))
{
  let project_configurers = [
    ProjectWriteConfiguration {
      name: String::from("CMake"),
      config_func: configure_cmake
    }
  ];

  for config in project_configurers {
    let config_name_str = config.name.as_str();
    before_write(config_name_str);

    let write_result = (config.config_func)(project_data);
    after_write((config_name_str, write_result));
  }
}