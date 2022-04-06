use std::{path::{Path, PathBuf}, rc::Rc};

use crate::{project_info::final_project_data::FinalProjectData, cli_config::CreateFilesCommand};
use self::{file_creation_info::{FileTypeGeneratingInfo, validate_which_generating, SharedFileInfo, validate_shared_file_info, FileGuardStyle}, code_file_writer::{write_code_files, extension_for, CodeFileType}};

mod code_file_writer;
mod file_creation_info;

pub fn handle_create_files(
  project_data: &Rc<FinalProjectData>,
  command: &CreateFilesCommand
) -> Result<(), String> {
  let which_generating: FileTypeGeneratingInfo = FileTypeGeneratingInfo::new(&command.file_types)?;
  validate_which_generating(&command.language, &which_generating)?;

  let shared_file_info: SharedFileInfo = SharedFileInfo::new(
    &command.file_name,
    project_data.get_project_root()
  );
  validate_shared_file_info(&shared_file_info)?;

  let file_guard: FileGuardStyle = if command.use_pragma_guards {
    FileGuardStyle::PragmaOnce
  }
  else {
    let guard_specifier_string: String = format!(
      "H_{}_{}",
      project_data.get_base_include_prefix(),
      &shared_file_info.shared_name
    )
      .to_uppercase()
      .replace('-', "_");

    FileGuardStyle::IncludeGuard(guard_specifier_string)
  };

  let maybe_existing_files = [
    (project_data.get_include_dir(), extension_for(CodeFileType::Header(command.language.clone()))),
    (project_data.get_src_dir(), extension_for(CodeFileType::Source(command.language.clone()))),
    (project_data.get_template_impl_dir(), extension_for(CodeFileType::TemplateImpl(command.language.clone()))),
  ]
    .map(|(code_root, extension)| format!(
      "{}/{}/{}{}",
      code_root,
      &shared_file_info.leading_dir_path,
      &shared_file_info.shared_name,
      extension
    ));

  for file_name in maybe_existing_files {
    if Path::new(&file_name).exists() {
      return Err(format!(
        "No files were created because file '{}' already exists and would be overwritten.",
        file_name
      ));
    }
  }

  let writer_result: Result<Vec<PathBuf>, _> = write_code_files(
    &which_generating,
    &shared_file_info,
    &file_guard,
    &project_data,
    &command.language
  );

  match writer_result {
    Ok(created_files) => {
      for file_path in created_files {
        println!(
          "Created: {}",
          file_path.to_str().unwrap()
        );
      }
    }
    Err(error) => return Err(error.to_string())
  }

  Ok(())
}