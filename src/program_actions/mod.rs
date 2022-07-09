mod create_project;
mod code_file_creator;
mod manage_dependencies;

pub use create_project::*;
pub use code_file_creator::*;
pub use manage_dependencies::*;

use crate::{project_info::{raw_data_in::dependencies::internal_dep_config::AllRawPredefinedDependencies, final_project_data::{FinalProjectData, UseableFinalProjectDataGroup, ProjectLoadFailureReason, ProjectConstructorConfig}, path_manipulation::absolute_path}};

pub fn parse_project_info(
  project_root_dir: &str,
  dep_config: &AllRawPredefinedDependencies,
  just_created_project_at: Option<String>
) -> Result<UseableFinalProjectDataGroup, ProjectLoadFailureReason> {
  FinalProjectData::new(
    project_root_dir,
    dep_config,
    ProjectConstructorConfig {
      just_created_library_project_at: just_created_project_at,
    }
  )
    .map_err(|failure_reason| failure_reason.map_message(|err_message|{
      format!(
        "When loading project using path '{}':\n{}",
        absolute_path(project_root_dir).unwrap().to_str().unwrap(),
        err_message
      )
    }))
}