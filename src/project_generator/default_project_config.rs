use std::{collections::{HashMap, HashSet}, iter::FromIterator};

use crate::project_info::raw_data_in::{RawProject, RawSubproject, ProjectLike, SpecificCompilerSpecifier, RawCompiledItem, CompiledItemType, BuildType, BuildConfigCompilerSpecifier, BuildConfig, SingleLanguageConfig, LanguageConfigMap};

use self::configuration::{MainFileLanguage, OutputLibType, CreationProjectOutputType};

pub mod configuration {
  #[derive(Clone, Copy)]
  pub enum MainFileLanguage {
    C,
    Cpp
  }

  pub enum OutputLibType {
    Static,
    Shared,
    ToggleStaticOrShared,
    HeaderOnly
  }

  pub enum CreationProjectOutputType {
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
  project_type: &CreationProjectOutputType,
  project_description: &str
) -> RawProject {
  RawProject {
    name: project_name.to_string(),
    include_prefix: include_prefix.to_owned(),
    description: String::from(project_description),
    version: String::from("0.0.1"),
    supported_compilers: HashSet::from_iter([
      SpecificCompilerSpecifier::GCC,
      SpecificCompilerSpecifier::Clang,
      SpecificCompilerSpecifier::MSVC,
    ]),
    prebuild_config: None,
    languages: LanguageConfigMap {
      C: SingleLanguageConfig {
        standard: 11
      },
      Cpp: SingleLanguageConfig {
        standard: 17
      }
    },
    output: HashMap::from_iter([
      (String::from("Main"), RawCompiledItem {
        entry_file: String::from(main_file_name(&project_lang, &project_type)),
        output_type: match project_type {
          CreationProjectOutputType::Executable => CompiledItemType::Executable,
          CreationProjectOutputType::Library(lib_type) => match lib_type {
            OutputLibType::Static => CompiledItemType::StaticLib,
            OutputLibType::Shared => CompiledItemType::SharedLib,
            OutputLibType::ToggleStaticOrShared => CompiledItemType::Library,
            OutputLibType::HeaderOnly => CompiledItemType::HeaderOnlyLib
          }
        },
        link: None,
        build_config: None
      })
    ]),
    predefined_dependencies: None,
    gcmake_dependencies: None,
    build_configs: HashMap::from_iter([
      (BuildType::Debug, HashMap::from_iter([
        (BuildConfigCompilerSpecifier::GCC, BuildConfig {
          compiler_flags: Some(create_string_set([ "-Og", "-g", "-Wall", "-Wextra", "-Wconversion", "-Wuninitialized", "-pedantic", "-pedantic-errors"])),
          linker_flags: None,
          defines: None
        }),
        (BuildConfigCompilerSpecifier::Clang, BuildConfig {
          compiler_flags: Some(create_string_set([ "-Og", "-g", "-Wall", "-Wextra", "-Wconversion", "-Wuninitialized", "-pedantic", "-pedantic-errors"])),
          linker_flags: None,
          defines: None
        }),
        (BuildConfigCompilerSpecifier::MSVC, BuildConfig {
          compiler_flags: Some(create_string_set([ "/Od", "/W4", "/DEBUG" ])),
          linker_flags: None,
          defines: None
        })
      ])),
      (BuildType::Release, HashMap::from_iter([
        (BuildConfigCompilerSpecifier::All, BuildConfig {
          compiler_flags: None,
          linker_flags: None,
          defines: Some(create_string_set(["NDEBUG"]))
        }),
        (BuildConfigCompilerSpecifier::GCC, BuildConfig {
          compiler_flags: Some(create_string_set([ "-O3", "-flto" ])),
          linker_flags: Some(create_string_set([ "-s" ])),
          defines: None
        }),
        (BuildConfigCompilerSpecifier::Clang, BuildConfig {
          compiler_flags: Some(create_string_set([ "-O3", "-flto" ])),
          linker_flags: Some(create_string_set([ "-s" ])),
          defines: None
        }),
        (BuildConfigCompilerSpecifier::MSVC, BuildConfig {
          compiler_flags: Some(create_string_set([ "/O2", "/GL" ])),
          linker_flags: None,
          defines: None
        })
      ])),
      (BuildType::MinSizeRel, HashMap::from_iter([
        (BuildConfigCompilerSpecifier::All, BuildConfig {
          compiler_flags: None,
          linker_flags: None,
          defines: Some(create_string_set(["NDEBUG"]))
        }),
        (BuildConfigCompilerSpecifier::GCC, BuildConfig {
          compiler_flags: Some(create_string_set([ "-Os", "-flto" ])),
          linker_flags: Some(create_string_set([ "-s" ])),
          defines: None
        }),
        (BuildConfigCompilerSpecifier::Clang, BuildConfig {
          compiler_flags: Some(create_string_set([ "-Os", "-flto" ])),
          linker_flags: Some(create_string_set([ "-s" ])),
          defines: None
        }),
        (BuildConfigCompilerSpecifier::MSVC, BuildConfig {
          compiler_flags: Some(create_string_set([ "/O1", "/GL" ])),
          linker_flags: None,
          defines: None
        })
      ])),
      (BuildType::RelWithDebInfo, HashMap::from_iter([
        (BuildConfigCompilerSpecifier::All, BuildConfig {
          compiler_flags: None,
          linker_flags: None,
          defines: Some(create_string_set(["NDEBUG"]))
        }),
        (BuildConfigCompilerSpecifier::GCC, BuildConfig {
          compiler_flags: Some(create_string_set([ "-O2", "-g" ])),
          linker_flags: None,
          defines: None
        }),
        (BuildConfigCompilerSpecifier::Clang, BuildConfig {
          compiler_flags: Some(create_string_set([ "-O2", "-g" ])),
          linker_flags: None,
          defines: None
        }),
        (BuildConfigCompilerSpecifier::MSVC, BuildConfig {
          compiler_flags: Some(create_string_set([ "/O2", "/DEBUG" ])),
          linker_flags: None,
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
  project_type: &CreationProjectOutputType,
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

pub fn main_file_name(project_lang: &MainFileLanguage, project_type: &CreationProjectOutputType) -> String {
  let extension_prefix: &str;
  let file_name: &str;

  match *project_type {
    CreationProjectOutputType::Executable => {
      extension_prefix = "c";
      file_name = "main";
    },
    CreationProjectOutputType::Library(_) => {
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