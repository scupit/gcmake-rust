use std::path::PathBuf;

use crate::{project_info::{path_manipulation::{relative_to_project_root, cleaned_path_str}, raw_data_in::LanguageConfigMap}, cli_config::clap_cli_config::FileCreationLang};

use super::code_file_writer::CodeFileType;
use colored::*;

#[derive(Clone)]
pub struct FileTypeGeneratingInfo {
  pub generating_header: bool,
  pub generating_source: bool,
  pub generating_template_impl: bool
}

impl FileTypeGeneratingInfo {
  pub fn new(specifier: &str) -> Result<Self, String> {
    let mut info = Self {
      generating_header: false,
      generating_source: false,
      generating_template_impl: false
    };

    for char in specifier.chars() {
      match char {
        'h' => info.generating_header = true,
        's' => info.generating_source = true,
        't' => info.generating_template_impl = true,
        invalid_char => return Err(format!(
          "Invalid character '{}' in file type specifier. Only 'h', 's', and 't' are allowed.",
          invalid_char
        ))
      }
    }

    return Ok(info);
  }

  pub fn will_generate_at_least_one(&self) -> bool {
    return self.generating_header || self.generating_source || self.generating_template_impl;
  }

  pub fn set_is_generating(&mut self, code_file_type: CodeFileType, should_generate: bool) {
    match code_file_type {
      CodeFileType::Header(_) => self.generating_header = should_generate,
      CodeFileType::Source(_) => self.generating_source = should_generate,
      CodeFileType::TemplateImpl(_) => self.generating_template_impl = should_generate
    }
  }

  pub fn get_is_generating(&self, code_file_type: CodeFileType) -> bool {
    match code_file_type {
      CodeFileType::Header(_) => self.generating_header,
      CodeFileType::Source(_) => self.generating_source,
      CodeFileType::TemplateImpl(_) => self.generating_template_impl
    }
  }
}

pub enum FileGuardStyle {
  IncludeGuard(String),
  PragmaOnce
}

impl FileGuardStyle {
  pub fn map_ident(
    &self,
    mapper_func: impl FnOnce(&str) -> String
  ) -> Self {
    return match self {
      Self::PragmaOnce => Self::PragmaOnce,
      Self::IncludeGuard(ident) => Self::IncludeGuard(mapper_func(ident))
    }
  }
}

pub struct SharedFileInfo {
  pub shared_name: String,
  pub shared_name_c_ident: String,
  pub leading_dir_path: String,
  pub cleaned_given_path: String
}

impl SharedFileInfo {
  pub fn new(
    file_class_name: &str,
    project_root: &str
  ) -> Self {
    let cleaned_given_path: String = relative_to_project_root(
      project_root,
      PathBuf::from(cleaned_path_str(file_class_name))
    ).to_str().unwrap().to_string();

    return if let Some(last_slash_index) = cleaned_given_path.rfind('/') {
      let shared_name: String = String::from(&cleaned_given_path[last_slash_index + 1..]); 
      let shared_name_c_ident: String = shared_name
        .replace(" ", "_")
        .replace("-", "_");

      Self {
        shared_name,
        shared_name_c_ident,
        leading_dir_path: String::from(&cleaned_given_path[0..last_slash_index]),
        cleaned_given_path
      }
    }
    else {
      let shared_name: String = cleaned_given_path.clone();
      let shared_name_c_ident: String = shared_name
        .replace(" ", "_")
        .replace("-", "_");

      Self {
        shared_name,
        shared_name_c_ident,
        cleaned_given_path,
        leading_dir_path: String::from("."),
      }
    }
  }
}

pub fn validate_which_generating(
  language_config_map: &LanguageConfigMap,
  lang: &FileCreationLang,
  which_generating: &FileTypeGeneratingInfo
) -> Result<(), String> {
  match *lang {
    FileCreationLang::C => {
      if which_generating.generating_template_impl {
        return Err(format!(
          "{}Cannot generate a template implementation file (.tpp) for the C language. Please remove 't' from the file types specifier.",
          "Error: ".red()
        ));
      }

      if language_config_map.c.is_none() {
        return Err(format!(
          "{}Can't generate a {} file for a project which doesn't support it. To fix this issue, add a configuration for the {} language in the project's root cmake_data.yaml.",
          "Error: ".red(),
          "C".yellow(),
          "C".yellow()
        ));
      }
    }
    FileCreationLang::Cpp | FileCreationLang::Cpp2 => {
      if language_config_map.cpp.is_none() {
        return Err(format!(
          "{}Can't generate a {} file for a project which doesn't support it. To fix this issue, add a configuration for the {} language in the project's root cmake_data.yaml.",
          "Error: ".red(),
          "C++ or cpp2".yellow(),
          "cpp".yellow()
        ));
      }
    },
    FileCreationLang::Cuda => {
      if language_config_map.cuda.is_none() {
        return Err(format!(
          "{}Can't generate a {} file for a project which doesn't support it. To fix this issue, add a configuration for the {} language in the project's root cmake_data.yaml.",
          "Error: ".red(),
          "CUDA".yellow(),
          "cuda".yellow()
        ));
      }
    }
  }

  Ok(())
}

pub fn validate_shared_file_info(shared_info: &SharedFileInfo) -> Result<(), String> {
  if shared_info.shared_name.contains('.') {
    return Err(format!(
      "Given file name '{}' should not have an extension, but does. Please remove the extension from {}",
      shared_info.shared_name,
      shared_info.cleaned_given_path
    ));
  }

  Ok(())
}
