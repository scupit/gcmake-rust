use crate::{project_info::{final_project_data::FinalProjectData, path_manipulation::cleaned_pathbuf}, cli_config::clap_cli_config::FileCreationLang};

use super::file_creation_info::{FileTypeGeneratingInfo, SharedFileInfo, FileGuardStyle};
use std::{io::{self, Write}, path::{PathBuf, Path}, fs::{self, File}};

pub fn write_code_files(
  which_generating: &FileTypeGeneratingInfo,
  shared_file_info: &SharedFileInfo,
  file_guard: &FileGuardStyle,
  project_info: &FinalProjectData,
  language: &FileCreationLang
) -> io::Result<Vec<PathBuf>> {
  let mut maybe_template_impl: Option<PathBuf> = None;
  let mut maybe_header: Option<PathBuf> = None;
  let mut maybe_source: Option<PathBuf> = None;

  if which_generating.generating_template_impl {
    maybe_template_impl = Some(
      write_template_impl(
        project_info,
        shared_file_info,
        language
      )?
    );
  }

  if which_generating.generating_header {
    maybe_header = Some(
      write_header(
        project_info,
        file_guard,
        shared_file_info,
        language,
        &maybe_template_impl
      )?
    );
  }

  if which_generating.generating_source {
    maybe_source = Some(
      write_source(
        project_info,
        shared_file_info,
        language,
        &maybe_header
      )?
    );
  }

  Ok(
    vec![maybe_header, maybe_source, maybe_template_impl]
      .into_iter()
      .filter(|item| item.is_some())
      .map(|item| item.unwrap())
      .collect()
  )
}

fn ensure_directory_structure_helper(code_dir: &str, leading_dir_structure: &str) -> io::Result<PathBuf> {
  let full_project_path = cleaned_pathbuf(
    Path::new(code_dir)
      .join(leading_dir_structure)
  );

  fs::create_dir_all(&full_project_path)?;
  Ok(full_project_path)
}

fn ensure_directory_structure(
  code_dir: &str,
  shared_file_info: &SharedFileInfo,
  extension_including_dot: &str
) -> io::Result<PathBuf> {
  let the_buf = ensure_directory_structure_helper(
    code_dir,
    &shared_file_info.leading_dir_path
  )?.join(format!("{}{}", &shared_file_info.shared_name, extension_including_dot));

  Ok(cleaned_pathbuf(the_buf))
}

pub enum CodeFileType {
  Header(FileCreationLang),
  Source(FileCreationLang),
  TemplateImpl(FileCreationLang)
}

pub fn extension_for(file_type: CodeFileType) -> &'static str {
  match file_type {
    CodeFileType::Header(lang) => match lang {
      FileCreationLang::C => ".h",
      FileCreationLang::Cpp => ".hpp"
    },
    CodeFileType::Source(lang) => match lang {
      FileCreationLang::C => ".c",
      FileCreationLang::Cpp => ".cpp"
    },
    CodeFileType::TemplateImpl(lang) => match lang {
      FileCreationLang::C => ".tpp",
      FileCreationLang::Cpp => ".tpp"
    }
  }
}

fn to_file_include_path(
  project: &FinalProjectData,
  file_including: &PathBuf
) -> String {
  let file_name: &str = file_including.file_name().unwrap().to_str().unwrap();
  return format!("{}/{}", project.get_full_include_prefix(), file_name);
}

fn write_header(
  project_info: &FinalProjectData,
  file_guard: &FileGuardStyle,
  file_info: &SharedFileInfo,
  language: &FileCreationLang,
  maybe_template_impl: &Option<PathBuf>
) -> io::Result<PathBuf> {
  // Ensure the directory structure exists
  let file_path = ensure_directory_structure(
    project_info.get_include_dir(),
    file_info,
    extension_for(CodeFileType::Header(language.clone()))
  )?;

  let header_file = File::create(&file_path)?;

  match file_guard {
    FileGuardStyle::IncludeGuard(specifier) => {
      writeln!(
        &header_file,
        "#ifndef {}\n#define {}",
        specifier,
        specifier
      )?;
    },
    FileGuardStyle::PragmaOnce => {
      writeln!(&header_file, "#pragma once")?;
    }
  }

  writeln!(&header_file, "\n\n")?;

  if let Some(template_impl_file) = maybe_template_impl {
    writeln!(
      &header_file,
      "#include \"{}\"",
      to_file_include_path(project_info, template_impl_file)
    )?;
  }

  if let FileGuardStyle::IncludeGuard(_) = file_guard {
    writeln!(&header_file, "#endif")?;
  }

  Ok(file_path)
}

fn write_source(
  project_info: &FinalProjectData,
  file_info: &SharedFileInfo,
  language: &FileCreationLang,
  maybe_header: &Option<PathBuf>
) -> io::Result<PathBuf> {
  // Ensure the directory structure exists
  let file_path = ensure_directory_structure(
    project_info.get_src_dir(),
    file_info,
    extension_for(CodeFileType::Source(language.clone()))
  )?;

  let source_file = File::create(&file_path)?;

  if let Some(header_file) = maybe_header {
    writeln!(
      &source_file,
      "#include \"{}\"",
      to_file_include_path(project_info, &header_file)
    )?;
  }

  Ok(file_path)
}

fn write_template_impl(
  project_info: &FinalProjectData,
  file_info: &SharedFileInfo,
  language: &FileCreationLang
) -> io::Result<PathBuf> {
  // Ensure the directory structure exists
  let file_path = ensure_directory_structure(
    project_info.get_template_impl_dir(),
    file_info,
    extension_for(CodeFileType::TemplateImpl(language.clone()))
  )?;

  let source_file = File::create(&file_path)?;

  writeln!(
    &source_file,
    "// Implement the template in {} here",
    file_path.to_str().unwrap()
  )?;

  Ok(file_path)
}