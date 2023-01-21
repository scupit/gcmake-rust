use crate::{project_info::{final_project_data::FinalProjectData, path_manipulation::cleaned_pathbuf, ProjectOutputType, CompiledOutputItem}, cli_config::clap_cli_config::FileCreationLang};

use super::file_creation_info::{FileTypeGeneratingInfo, SharedFileInfo, FileGuardStyle};
use std::{io::{self, Write}, path::{PathBuf, Path}, fs::{self, File}};

pub fn write_code_files(
  which_generating: &FileTypeGeneratingInfo,
  shared_file_info: &SharedFileInfo,
  file_guard: &FileGuardStyle,
  project_info: &FinalProjectData,
  language: &FileCreationLang,
  is_private: bool
) -> io::Result<Vec<PathBuf>> {
  let mut maybe_template_impl: Option<PathBuf> = None;
  let mut maybe_header: Option<PathBuf> = None;
  let mut maybe_source: Option<PathBuf> = None;

  if which_generating.generating_template_impl {
    maybe_template_impl = Some(
      write_template_impl(
        project_info,
        &file_guard.map_ident(|ident| format!("T_IMPL_{}", ident)),
        shared_file_info,
        language,
        is_private
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
        &maybe_template_impl,
        is_private
      )?
    );
  }

  if which_generating.generating_source {
    maybe_source = Some(
      write_source(
        project_info,
        shared_file_info,
        language,
        &maybe_header,
        is_private
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

#[derive(Clone)]
pub enum CodeFileType {
  Header(FileCreationLang),
  Source(FileCreationLang),
  TemplateImpl(FileCreationLang)
}

pub fn extension_for(file_type: CodeFileType, is_private: bool) -> &'static str {
  match file_type {
    CodeFileType::Header(lang) => match lang {
      FileCreationLang::C => {
        if is_private { ".private.h" }
        else { ".h" }
      },
      FileCreationLang::Cpp
        | FileCreationLang::Cpp2 => {
        if is_private { ".private.hpp" }
        else { ".hpp" }
      }
    },
    CodeFileType::Source(lang) => match lang {
      FileCreationLang::C => ".c",
      FileCreationLang::Cpp => ".cpp",
      FileCreationLang::Cpp2 => ".cpp2"
    },
    CodeFileType::TemplateImpl(lang) => match lang {
      FileCreationLang::C => "IGNORED",
      FileCreationLang::Cpp
        | FileCreationLang::Cpp2 => {
        if is_private { ".private.tpp" }
        else { ".tpp" }
      }
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
  maybe_template_impl: &Option<PathBuf>,
  is_private: bool
) -> io::Result<PathBuf> {
  let container_dir: &str = if is_private
    { project_info.get_src_dir_relative_to_cwd() }
    else { project_info.get_include_dir_relative_to_cwd() };

  // Ensure the directory structure exists
  let file_path = ensure_directory_structure(
    container_dir,
    file_info,
    extension_for(CodeFileType::Header(language.clone()), is_private)
  )?;

  let mut header_file: File = File::create(&file_path)?;
  write_include_guard_begin(&mut header_file, file_guard)?;

  match project_info.get_project_output_type() {
    ProjectOutputType::CompiledLibProject => {
      write_compiled_lib_header_section(
        project_info,
        file_info,
        language,
        &mut header_file,
        is_private
      )?;
    },
    ProjectOutputType::ExeProject | ProjectOutputType::HeaderOnlyLibProject => match language {
      FileCreationLang::C => {
        writeln!(
          header_file,
          "\nint placeholder_{}(void);\n",
          &file_info.shared_name_c_ident
        )?;
      },
      FileCreationLang::Cpp => {
        writeln!(
          header_file,
          "\nclass {}\n{{\n\tpublic:\n\t\tvoid printName();\n}};\n",
          &file_info.shared_name_c_ident
        )?;
      },
      FileCreationLang::Cpp2 => {
        writeln!(
          header_file,
          "\nint placeholder_{}(const int);\n",
          &file_info.shared_name_c_ident
        )?;
      }
    }
  }

  if let Some(template_impl_file) = maybe_template_impl {
    writeln!(
      &header_file,
      "#include \"{}\"\n",
      to_file_include_path(project_info, template_impl_file)
    )?;
  }

  write_include_guard_end(&mut header_file, file_guard)?;

  Ok(file_path)
}

fn write_compiled_lib_header_section(
  project_info: &FinalProjectData,
  file_info: &SharedFileInfo,
  language: &FileCreationLang,
  header_file: &mut File,
  is_private: bool
) -> io::Result<()> {
  // This is guaranteed to work because library projects can only build one library.
  let (output_name, _) = project_info.get_outputs().iter().nth(0).unwrap();

  if !is_private {
    writeln!(
      header_file,
      "#include \"{}\"",
      CompiledOutputItem::export_macro_header_include_path(
        project_info.get_full_include_prefix(),
        output_name
      )
    )?;
  }

  let export_macro: String = if is_private
    { String::from("") }
    else { format!("{} ", CompiledOutputItem::str_export_macro(output_name)) };

  match language {
    FileCreationLang::C => {
      writeln!(
        header_file,
        "\n{}int placeholder_{}(void);\n",
        export_macro,
        &file_info.shared_name_c_ident
      )?;
    },
    FileCreationLang::Cpp => {
      writeln!(
        header_file,
        "\nclass {}{}\n{{\n\tpublic:\n\t\tvoid printName();\n}};\n",
        export_macro,
        &file_info.shared_name_c_ident
      )?;
    },
    FileCreationLang::Cpp2 => {
      writeln!(
        header_file,
        "\n{}int placeholder_{}(const int);\n",
        export_macro,
        &file_info.shared_name_c_ident
      )?;
    }
  }

  Ok(())
}

fn write_source(
  project_info: &FinalProjectData,
  file_info: &SharedFileInfo,
  language: &FileCreationLang,
  maybe_header: &Option<PathBuf>,
  is_private: bool
) -> io::Result<PathBuf> {
  // Ensure the directory structure exists
  let file_path = ensure_directory_structure(
    project_info.get_src_dir_relative_to_cwd(),
    file_info,
    extension_for(CodeFileType::Source(language.clone()), is_private)
  )?;

  let source_file = File::create(&file_path)?;

  if let Some(header_file) = maybe_header {
    writeln!(
      &source_file,
      "#include \"{}\"\n",
      to_file_include_path(project_info, &header_file)
    )?;

    match language {
      FileCreationLang::C => {
        writeln!(
          &source_file,
          "int placeholder_{}(const int n) {{\n\treturn n * 2;\n}}",
          &file_info.shared_name_c_ident
        )?;
      },
      FileCreationLang::Cpp => {
        writeln!(
          &source_file,
          "#include <iostream>\n\nvoid {}::printName() {{\n\tstd::cout << \"{}\\n\";\n}}",
          &file_info.shared_name_c_ident,
          &file_info.shared_name_c_ident
        )?;
      },
      FileCreationLang::Cpp2 => {
        writeln!(
          &source_file,
          "placeholder_{}: (n: int) -> int = {{\n\treturn n * 2;\n}}",
          &file_info.shared_name_c_ident
        )?;
      }
    }
  }

  Ok(file_path)
}

fn write_template_impl(
  project_info: &FinalProjectData,
  file_guard: &FileGuardStyle,
  file_info: &SharedFileInfo,
  language: &FileCreationLang,
  is_private: bool
) -> io::Result<PathBuf> {
  let container_dir: &str = if is_private
    { project_info.get_src_dir_relative_to_cwd() }
    else { project_info.get_include_dir_relative_to_cwd() };

  // Ensure the directory structure exists
  let file_path = ensure_directory_structure(
    container_dir,
    file_info,
    extension_for(CodeFileType::TemplateImpl(language.clone()), is_private)
  )?;

  let mut template_impl_file = File::create(&file_path)?;
  write_include_guard_begin(&mut template_impl_file, file_guard)?;

  writeln!(
    &template_impl_file,
    "// Implement the template in {} here",
    file_path.to_str().unwrap()
  )?;

  write_include_guard_end(&mut template_impl_file, file_guard)?;

  Ok(file_path)
}

fn write_include_guard_begin(
  out_file: &mut File,
  file_guard: &FileGuardStyle
) -> io::Result<()> {
  match file_guard {
    FileGuardStyle::IncludeGuard(specifier) => {
      writeln!(
        out_file,
        "#ifndef {}\n#define {}\n",
        specifier,
        specifier
      )?;
    },
    FileGuardStyle::PragmaOnce => {
      writeln!(out_file, "#pragma once\n")?;
    }
  }

  Ok(())
}

fn write_include_guard_end(
  out_file: &mut File,
  file_guard: &FileGuardStyle
) -> io::Result<()> {
  if let FileGuardStyle::IncludeGuard(_) = file_guard {
    writeln!(out_file, "#endif")?;
  }

  Ok(())
}
