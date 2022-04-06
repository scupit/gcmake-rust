mod create_project;
mod code_file_creator;

pub use create_project::*;
pub use code_file_creator::*;

use crate::{project_info::{raw_data_in::dependencies::internal_dep_config::AllPredefinedDependencies, final_project_data::{FinalProjectData, UseableFinalProjectDataGroup, ProjectLoadFailureReason}}};

pub fn parse_project_info(
  project_root_dir: &str,
  dep_config: &AllPredefinedDependencies
) -> Result<UseableFinalProjectDataGroup, ProjectLoadFailureReason> {
  FinalProjectData::new(project_root_dir, dep_config)
}
