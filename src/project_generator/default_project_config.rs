use std::{collections::{HashMap, HashSet}, iter::FromIterator};

use crate::{project_info::{raw_data_in::{RawProject, RawSubproject, SpecificCompilerSpecifier, RawCompiledItem, OutputItemType, BuildType, BuildConfigCompilerSpecifier, BuildConfig, SingleLanguageConfig, LanguageConfigMap, RawTestProject}, base_include_prefix_for_test}, program_actions::ProjectTypeCreating};

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

  impl OutputLibType {
    pub fn is_compiled_lib(&self) -> bool {
      return match self {
        Self::HeaderOnly => false,
        _ => true
      }
    }
  }

  pub enum CreationProjectOutputType {
    Library(OutputLibType),
    Executable
  }
}

pub struct CreatedProject {
  pub name: String,
  pub info: DefaultProjectInfo
}

pub enum DefaultProjectInfo {
  RootProject(RawProject),
  Subproject(RawSubproject),
  TestProject(RawTestProject)
}

pub fn get_default_project_config(
  project_name: &str,
  include_prefix: &str,
  project_lang: &MainFileLanguage,
  project_output_type: &CreationProjectOutputType,
  _project_type_creating: &ProjectTypeCreating,
  project_description: &str,
  project_vendor: &str,
  requires_custom_main: Option<bool>
) -> RawProject {
  RawProject {
    name: project_name.to_string(),
    include_prefix: include_prefix.to_string(),
    description: String::from(project_description),
    vendor: String::from(project_vendor),
    version: String::from("0.0.1"),
    supported_compilers: HashSet::from_iter([
      SpecificCompilerSpecifier::GCC,
      SpecificCompilerSpecifier::Clang,
      SpecificCompilerSpecifier::MSVC,
    ]),
    prebuild_config: None,
    languages: LanguageConfigMap {
      c: SingleLanguageConfig {
        standard: 11
      },
      cpp: SingleLanguageConfig {
        standard: 17
      }
    },
    output: HashMap::from_iter([
      (format!("{}", project_name), RawCompiledItem {
        entry_file: String::from(main_file_name(project_name, &project_lang, &project_output_type)),
        output_type: match project_output_type {
          CreationProjectOutputType::Executable => OutputItemType::Executable,
          CreationProjectOutputType::Library(lib_type) => match lib_type {
            OutputLibType::Static => OutputItemType::StaticLib,
            OutputLibType::Shared => OutputItemType::SharedLib,
            OutputLibType::ToggleStaticOrShared => OutputItemType::CompiledLib,
            OutputLibType::HeaderOnly => OutputItemType::HeaderOnlyLib
          }
        },
        link: None,
        build_config: None,
        requires_custom_main
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
    subprojects: None,
    test_framework: None
  }
}

pub fn get_default_subproject_config(
  project_name: &str,
  include_prefix: &str,
  project_lang: &MainFileLanguage,
  project_output_type: &CreationProjectOutputType,
  project_type_creaing: &ProjectTypeCreating,
  project_description: &str,
  requires_custom_main: Option<bool>
) -> RawSubproject {
  RawSubproject::from(
    get_default_project_config(
      project_name,
      include_prefix,
      project_lang,
      project_output_type,
      project_type_creaing,
      project_description,
      "VENDOR FIELD NOT USED FOR SUBPROJECTS",
      requires_custom_main
    )
  )
}

pub fn get_default_test_project_config(
  project_name: &str,
  include_prefix: &str,
  project_lang: &MainFileLanguage,
  project_output_type: &CreationProjectOutputType,
  project_type_creaing: &ProjectTypeCreating,
  project_description: &str,
  requires_custom_main: Option<bool>
) -> RawTestProject {
  RawTestProject::from(RawSubproject::from(
    get_default_project_config(
      project_name,
      include_prefix,
      project_lang,
      project_output_type,
      project_type_creaing,
      project_description,
      "VENDOR FIELD NOT USED FOR TEST PROJECTS",
      requires_custom_main
    )
  ))
}

pub fn main_file_name(
  project_name: &str,
  project_lang: &MainFileLanguage,
  project_type: &CreationProjectOutputType
) -> String {
  let extension_prefix: &str;
  let file_name: &str;

  match *project_type {
    CreationProjectOutputType::Executable => {
      extension_prefix = "c";
      file_name = "main";
    },
    CreationProjectOutputType::Library(_) => {
      extension_prefix = "h";
      file_name = project_name;
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