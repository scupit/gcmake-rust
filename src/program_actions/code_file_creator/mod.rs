use std::{path::{Path, PathBuf}, rc::Rc};

use crate::{project_info::{final_project_data::FinalProjectData, path_manipulation::{relative_to_project_root, absolute_path}}, cli_config::clap_cli_config::CreateFilesCommand, common::prompt::{prompt_until_custom}};
use self::{file_creation_info::{FileTypeGeneratingInfo, validate_which_generating, SharedFileInfo, validate_shared_file_info, FileGuardStyle}, code_file_writer::{write_code_files, extension_for, CodeFileType}};

mod code_file_writer;
mod file_creation_info;
mod file_creation_prompts;
use colored::*;

pub use file_creation_prompts::prompt_for_initial_compiled_lib_file_pair_name;

enum FileCollisionHandleOption {
  Unspecified,
  SkipOne,
  OverwriteOne,
  CancelRest,
  ReplaceAll
}

pub fn handle_create_files(
  project_data: &Rc<FinalProjectData>,
  command: &CreateFilesCommand
) -> Result<bool, String> {
  let which_generating: FileTypeGeneratingInfo = FileTypeGeneratingInfo::new(&command.which)?;
  validate_which_generating(project_data.get_language_info(), &command.language, &which_generating)?;

  let mut global_file_collision_option = FileCollisionHandleOption::Unspecified;

  for relative_file_name in &command.relative_file_names {
    create_single_file_set(
      &mut global_file_collision_option,
      project_data,
      command,
      which_generating.clone(),
      relative_file_name
    )?;

    if let FileCollisionHandleOption::CancelRest = &global_file_collision_option {
      println!(
        "{}",
        "\nCancelled creating the rest of the code files. No more will be created.\n".green()
      );
      return Ok(true);
    }
  }

  Ok(false)
}

fn create_single_file_set(
  global_file_collision_option: &mut FileCollisionHandleOption,
  project_data: &Rc<FinalProjectData>,
  command: &CreateFilesCommand,
  mut which_generating: FileTypeGeneratingInfo,
  file_name: &str
) -> Result<(), String> {
  let shared_file_info: SharedFileInfo = SharedFileInfo::new(
    &file_name,
    project_data.get_project_root_relative_to_cwd()
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

  let used_include_path: &Path = if command.should_files_be_private
    { project_data.get_src_dir_relative_to_cwd() }
    else { project_data.get_include_dir_relative_to_cwd() };

  let maybe_existing_files = [
    // (project_data.get_include_dir(), extension_for(CodeFileType::Header(command.language.clone()))),
    // (project_data.get_src_dir(), extension_for(CodeFileType::Source(command.language.clone()))),
    // (project_data.get_template_impl_dir(), extension_for(CodeFileType::TemplateImpl(command.language.clone()))),
    (used_include_path, CodeFileType::Header(command.language.clone())),
    (project_data.get_src_dir_relative_to_cwd(), CodeFileType::Source(command.language.clone())),
    (used_include_path, CodeFileType::TemplateImpl(command.language.clone()))
  ]
    .map(|(code_root, code_file_type)| {
      let file_path: PathBuf = code_root
        .join(&shared_file_info.leading_dir_path)
        .join(format!(
          "{}{}",
          &shared_file_info.shared_name,
          extension_for(code_file_type.clone(), command.should_files_be_private)
        ));

      (file_path, code_file_type)
    });

  for (file_path, ref code_file_type) in maybe_existing_files {
    match global_file_collision_option {
      FileCollisionHandleOption::ReplaceAll => continue,
      FileCollisionHandleOption::CancelRest => {
        which_generating.set_is_generating(code_file_type.clone(), false);
        continue;
      },
      _ => ()
    }

    let is_file_about_to_be_written: bool = which_generating.get_is_generating(code_file_type.clone());

    if is_file_about_to_be_written && file_path.exists() {
      let file_path_absolute: PathBuf = absolute_path(file_path)?;
      let file_path_relative_to_working_dir: &Path = file_path_absolute.strip_prefix(project_data.get_absolute_project_root()).unwrap();

      let local_collision_mode: FileCollisionHandleOption = prompt_until_custom(
        &format!(
          "\nFile '{}' already exists.\n[{}]kip it, [{}]verwrite it, [{}]ancel rest, or replace [{}]ll?",
          file_path_relative_to_working_dir.to_str().unwrap().yellow(),
          "s".cyan(),
          "o".cyan(),
          "c".cyan(),
          "a".cyan()
        ),
        |value| match value {
          "s" => Some(FileCollisionHandleOption::SkipOne),
          "o" => Some(FileCollisionHandleOption::OverwriteOne),
          "c" => Some(FileCollisionHandleOption::CancelRest),
          "a" => Some(FileCollisionHandleOption::ReplaceAll),
          _ => None
        }
      ).map_err(|io_err| io_err.to_string())?;

      match local_collision_mode {
        FileCollisionHandleOption::Unspecified => unreachable!(),
        FileCollisionHandleOption::SkipOne => which_generating.set_is_generating(code_file_type.clone(), false),
        FileCollisionHandleOption::CancelRest => {
          *global_file_collision_option = FileCollisionHandleOption::CancelRest;
          which_generating.set_is_generating(code_file_type.clone(), false)
        },
        FileCollisionHandleOption::OverwriteOne => (),
        FileCollisionHandleOption::ReplaceAll => *global_file_collision_option = FileCollisionHandleOption::ReplaceAll
      }
    }
  }

  if which_generating.will_generate_at_least_one() {
    let writer_result: Result<Vec<PathBuf>, _> = write_code_files(
      &which_generating,
      &shared_file_info,
      &file_guard,
      &project_data,
      &command.language,
      command.should_files_be_private
    );

    match writer_result {
      Ok(created_files) => {
        for file_path in created_files {
          println!(
            "Created: {}",
            relative_to_project_root(&project_data.get_project_root_relative_to_cwd(), file_path).to_str().unwrap().cyan()
          );
        }
      }
      Err(error) => return Err(error.to_string())
    }
  }

  Ok(())
}