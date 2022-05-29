mod create_project;
mod code_file_creator;
mod manage_dependencies;

pub use create_project::*;
pub use code_file_creator::*;
pub use manage_dependencies::{local_dep_config_repo_location, update_dependency_config_repo, DepConfigUpdateResult};

use crate::{project_info::{raw_data_in::dependencies::internal_dep_config::AllPredefinedDependencies, final_project_data::{FinalProjectData, UseableFinalProjectDataGroup, ProjectLoadFailureReason}}};

pub fn parse_project_info(
  project_root_dir: &str,
  dep_config: &AllPredefinedDependencies
) -> Result<UseableFinalProjectDataGroup, ProjectLoadFailureReason> {
  FinalProjectData::new(project_root_dir, dep_config)
}