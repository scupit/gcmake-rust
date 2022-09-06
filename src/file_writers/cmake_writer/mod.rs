mod cmake_utils_writer;
mod cmakelists_writer;
mod cmake_writer_helpers;
mod ordered_utils;

use std::{io};

use crate::project_info::{dependency_graph_mod::dependency_graph::{DependencyGraphInfoWrapper}};

use self::cmakelists_writer::configure_cmake_helper;

pub fn configure_cmake<'a>(root_graph_info: &'a DependencyGraphInfoWrapper<'a>) -> io::Result<()> {
  configure_cmake_helper(&root_graph_info.root_dep_graph, &root_graph_info.sorted_info)?;
  Ok(())
}
