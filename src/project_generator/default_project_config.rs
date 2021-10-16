use std::{collections::{HashMap, HashSet}, error::Error, fs::create_dir, io::{self, stdin}, iter::FromIterator, path::{Path, PathBuf}};
use crate::{data_types::raw_types::{BuildConfig, BuildConfigCompilerSpecifier, BuildType, CompiledItemType, CompilerSpecifier, ImplementationLanguage, LanguageConfig, ProjectLike, RawCompiledItem, RawProject, RawSubproject}, main};
use self::configuration::{MainFileLanguage, OutputLibType, ProjectOutputType};


pub mod configuration {
  #[derive(Clone, Copy)]
  pub enum MainFileLanguage {
    C,
    Cpp
  }

  pub enum OutputLibType {
    Static,
    Shared
  }

  pub enum ProjectOutputType {
    Library(OutputLibType),
    Executable
  }
}

pub enum DefaultProject {
  MainProject(RawProject),
  Subproject(RawSubproject)
}

impl DefaultProject {
  pub fn unwrap_projectlike(&self) -> Box<&dyn ProjectLike> {
    match self {
      Self::MainProject(data) => Box::new(data),
      Self::Subproject(data) => Box::new(data)
    }
  }
}

pub fn get_default_project_config(
  project_name: &str,
  include_prefix: &str,
  project_lang: &MainFileLanguage,
  project_type: &ProjectOutputType,
  project_description: &str
) -> RawProject {

  RawProject {
      name: project_name.to_string(),
      include_prefix: include_prefix.to_owned(),
      description: String::from(project_description),
      version: String::from("0.0.1"),
      supported_compilers: HashSet::from_iter([
        CompilerSpecifier::GCC,
        CompilerSpecifier::Clang,
        CompilerSpecifier::MSVC,
      ]),
      languages: HashMap::from_iter([
        (ImplementationLanguage::C, LanguageConfig {
          allowed_standards: HashSet::from_iter([99, 11, 17]),
          default_standard: 99
        }),
        (ImplementationLanguage::Cpp, LanguageConfig {
          allowed_standards: HashSet::from_iter([11, 14, 17, 20]),
          default_standard: 17
        })
      ]),
      output: HashMap::from_iter([
        (String::from("Main"), RawCompiledItem {
          entry_file: String::from(main_file_name(&project_lang, &project_type)),
          output_type: match project_type {
            ProjectOutputType::Executable => CompiledItemType::Executable,
            // TODO: Allow the library type to be selected once type selection is implemented
            ProjectOutputType::Library(lib_type) => match lib_type {
              OutputLibType::Static => CompiledItemType::StaticLib,
              OutputLibType::Shared => CompiledItemType::SharedLib
            }
          },
          link: None
        })
      ]),
      build_configs: HashMap::from_iter([
        (BuildType::Debug, HashMap::from_iter([
          (BuildConfigCompilerSpecifier::GCC, BuildConfig {
            flags: Some(create_string_set([ "-Og", "-g", "-Wall", "-Wextra", "-Wconversion", "-Wuninitialized", "-pedantic", "-pedantic-errors"])),
            defines: None
          }),
          (BuildConfigCompilerSpecifier::Clang, BuildConfig {
            flags: Some(create_string_set([ "-Og", "-g", "-Wall", "-Wextra", "-Wconversion", "-Wuninitialized", "-pedantic", "-pedantic-errors"])),
            defines: None
          }),
          (BuildConfigCompilerSpecifier::MSVC, BuildConfig {
            flags: Some(create_string_set([ "/Od", "/W4", "/DEBUG" ])),
            defines: None
          })
        ])),
        (BuildType::Release, HashMap::from_iter([
          (BuildConfigCompilerSpecifier::All, BuildConfig {
            flags: None,
            defines: Some(create_string_set(["NDEBUG"]))
          }),
          (BuildConfigCompilerSpecifier::GCC, BuildConfig {
            flags: Some(create_string_set([ "-O3", "-s"])),
            defines: None
          }),
          (BuildConfigCompilerSpecifier::Clang, BuildConfig {
            flags: Some(create_string_set([ "-O3", "-Wl,-s"])),
            defines: None
          }),
          (BuildConfigCompilerSpecifier::MSVC, BuildConfig {
            flags: Some(create_string_set([ "/O2" ])),
            defines: None
          })
        ])),
        (BuildType::MinSizeRel, HashMap::from_iter([
          (BuildConfigCompilerSpecifier::All, BuildConfig {
            flags: None,
            defines: Some(create_string_set(["NDEBUG"]))
          }),
          (BuildConfigCompilerSpecifier::GCC, BuildConfig {
            flags: Some(create_string_set([ "-Os", "-s"])),
            defines: None
          }),
          (BuildConfigCompilerSpecifier::Clang, BuildConfig {
            flags: Some(create_string_set([ "-Os", "-Wl,-s"])),
            defines: None
          }),
          (BuildConfigCompilerSpecifier::MSVC, BuildConfig {
            flags: Some(create_string_set([ "/O1" ])),
            defines: None
          })
        ])),
        (BuildType::RelWithDebInfo, HashMap::from_iter([
          (BuildConfigCompilerSpecifier::All, BuildConfig {
            flags: None,
            defines: Some(create_string_set(["NDEBUG"]))
          }),
          (BuildConfigCompilerSpecifier::GCC, BuildConfig {
            flags: Some(create_string_set([ "-O2", "-g"])),
            defines: None
          }),
          (BuildConfigCompilerSpecifier::Clang, BuildConfig {
            flags: Some(create_string_set([ "-O2", "-g"])),
            defines: None
          }),
          (BuildConfigCompilerSpecifier::MSVC, BuildConfig {
            flags: Some(create_string_set([ "/O2", "/DEBUG" ])),
            defines: None
          })
        ]))
      ]),
      default_build_type: BuildType::Debug,
      global_defines: None,
      subprojects: None
    }
}

pub fn get_default_subproject_config(
  project_name: &str,
  include_prefix: &str,
  project_lang: &MainFileLanguage,
  project_type: &ProjectOutputType,
  project_description: &str
) -> RawSubproject {
  RawSubproject::from(
    get_default_project_config(
      project_name,
      include_prefix,
      project_lang,
      project_type,
      project_description
    )
  )
}

pub fn main_file_name(project_lang: &MainFileLanguage, project_type: &ProjectOutputType) -> String {
  let extension_prefix: &str;
  let file_name: &str;

  match *project_type {
    ProjectOutputType::Executable => {
      extension_prefix = "c";
      file_name = "main";
    },
    ProjectOutputType::Library(_) => {
      extension_prefix = "h";
      file_name = "lib";
    }
  };

  let extension_suffix = match *project_lang {
    MainFileLanguage::C => "",
    MainFileLanguage::Cpp => "pp"
  };

  return format!("{}.{}{}", file_name, extension_prefix, extension_suffix);
}

fn create_string_set<const AMOUNT: usize>(arr: [&str; AMOUNT]) -> HashSet<String> {
  return arr
    .iter()
    .map(|&borrowed_str| String::from(borrowed_str))
    .collect()
}