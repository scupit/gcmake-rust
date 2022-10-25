mod cmake_writer;
mod debian_sh_install_writer;

use std::{io::{self}};
use crate::project_info::{dependency_graph_mod::dependency_graph::{DependencyGraphInfoWrapper}};

pub struct ProjectWriteConfiguration<'a> {
  name: String,
  config_func: fn(&'a DependencyGraphInfoWrapper<'a>) -> io::Result<()>,
}

pub fn write_configurations<'a, FBefore, FAfter>(
  root_graph_info: &'a DependencyGraphInfoWrapper<'a>,
  before_write: FBefore,
  after_write: FAfter
) -> io::Result<()>
  where
    FBefore: Fn(&str),
    FAfter: Fn((&str, io::Result<()>))
{
  let project_configurers = [
    ProjectWriteConfiguration {
      name: String::from("CMake"),
      config_func: cmake_writer::configure_cmake
    },
    ProjectWriteConfiguration {
      name: String::from("Debian dev dependency install sh"),
      config_func: debian_sh_install_writer::write_debian_dep_install_sh
    }
  ];

  for config in project_configurers {
    let config_name_str = config.name.as_str();
    before_write(config_name_str);

    let write_result = (config.config_func)(root_graph_info);
    after_write((config_name_str, write_result));
  }

  Ok(())
}
