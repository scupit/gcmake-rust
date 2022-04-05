mod create_project;
mod code_file_creator;

pub use create_project::*;
pub use code_file_creator::*;

use crate::{project_info::{raw_data_in::dependencies::internal_dep_config::AllPredefinedDependencies, final_project_data::{FinalProjectData, UseableFinalProjectDataGroup}, final_dependencies::FinalGitRepoDescriptor}, logger::exit_error_log, project_generator::{configuration::{MainFileLanguage, ProjectOutputType, OutputLibType}, create_project_at}};

pub fn get_project_info_then<F>(
  project_root_dir: &str,
  dep_config: &AllPredefinedDependencies,
  on_parse_success: F
)
  where F: FnOnce(UseableFinalProjectDataGroup)
{
  match FinalProjectData::new(project_root_dir, dep_config) {
    Ok(project_data_group) => on_parse_success(project_data_group),
    Err(message) => exit_error_log(&message)
  }
}
