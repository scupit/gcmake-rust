mod cmake_utils_writer;
mod cmakelists_writer;
mod cmake_writer_helpers;
mod ordered_utils;

use std::{io};

use crate::project_info::{final_project_data::UseableFinalProjectDataGroup, dependency_graph_mod::dependency_graph::{DependencyGraphInfoWrapper, DependencyGraph}};

use self::cmakelists_writer::configure_cmake_helper;

pub fn configure_cmake(root_graph_info: &DependencyGraphInfoWrapper) -> io::Result<()> {
  configure_cmake_helper(&root_graph_info.root_dep_graph, &root_graph_info.sorted_info)?;
  Ok(())
}
