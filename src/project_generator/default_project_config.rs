use std::{collections::{HashMap, BTreeSet, BTreeMap}, iter::FromIterator};

use crate::{project_info::{raw_data_in::{RawProject, RawSubproject, SpecificCompilerSpecifier, RawCompiledItem, OutputItemType, BuildType, BuildConfigCompilerSpecifier, RawBuildConfig, SingleLanguageConfig, LanguageConfigMap, RawTestProject, RawGlobalPropertyConfig, DefaultCompiledLibType}}, program_actions::ProjectTypeCreating};

use self::configuration::{MainFileLanguage, OutputLibType, CreationProjectOutputType};

pub mod configuration {
  #[derive(Clone, Copy)]
  pub enum MainFileLanguage {
    C,
    Cpp
  }

  #[derive(Clone)]
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

  #[derive(Clone)]
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

fn should_support_emscripten(project_type_creating: &ProjectTypeCreating) -> bool {
  return match project_type_creating {
    ProjectTypeCreating::RootProject { include_emscripten_support: true } => true,
    _ => false
  }
}

fn supported_compilers_default(include_emscripten_support: bool) -> BTreeSet<SpecificCompilerSpecifier> {
  let mut supported_compilers: BTreeSet<SpecificCompilerSpecifier> = BTreeSet::from_iter([
    SpecificCompilerSpecifier::GCC,
    SpecificCompilerSpecifier::Clang,
    SpecificCompilerSpecifier::MSVC
  ]);

  if include_emscripten_support {
    supported_compilers.insert(SpecificCompilerSpecifier::Emscripten);
  }

  return supported_compilers;
}

type BuildConfigByCompiler = BTreeMap<BuildConfigCompilerSpecifier, RawBuildConfig>;

fn build_configs_debug_default(include_emscripten_support: bool) -> BuildConfigByCompiler {
  let mut debug_config: BuildConfigByCompiler = BTreeMap::from_iter([
    (BuildConfigCompilerSpecifier::GCC, RawBuildConfig {
      compiler_flags: Some(create_string_set([ "-Og", "-g", "-Wall", "-Wextra", "-Wconversion", "-Wuninitialized", "-pedantic", "-pedantic-errors"])),
      link_time_flags: None,
      linker_flags: None,
      defines: None
    }),
    (BuildConfigCompilerSpecifier::Clang, RawBuildConfig {
      compiler_flags: Some(create_string_set([ "-Og", "-g", "-Wall", "-Wextra", "-Wconversion", "-Wuninitialized", "-pedantic", "-pedantic-errors"])),
      link_time_flags: None,
      linker_flags: None,
      defines: None
    }),
    (BuildConfigCompilerSpecifier::MSVC, RawBuildConfig {
      compiler_flags: Some(create_string_set([ "/Od", "/W4", "/DEBUG" ])),
      link_time_flags: None,
      linker_flags: None,
      defines: None
    })
  ]);

  if include_emscripten_support {
    debug_config.insert(
      BuildConfigCompilerSpecifier::Emscripten,
      RawBuildConfig {
        compiler_flags: Some(create_string_set([ "-O0", "-g", "-gsource-map" ])),
        link_time_flags: None,
        linker_flags: None,
        defines: None
      }
    );
  }

  return debug_config;
}

fn build_configs_release_default(include_emscripten_support: bool) -> BuildConfigByCompiler {
  let mut release_config: BuildConfigByCompiler = BTreeMap::from_iter([
    (BuildConfigCompilerSpecifier::AllCompilers, RawBuildConfig {
      compiler_flags: None,
      link_time_flags: None,
      linker_flags: None,
      defines: Some(create_string_set(["NDEBUG"]))
    }),
    (BuildConfigCompilerSpecifier::GCC, RawBuildConfig {
      compiler_flags: Some(create_string_set([ "-O3" ])),
      link_time_flags: None,
      linker_flags: Some(create_string_set([ "-s" ])),
      defines: None
    }),
    (BuildConfigCompilerSpecifier::Clang, RawBuildConfig {
      compiler_flags: Some(create_string_set([ "-O3" ])),
      link_time_flags: None,
      linker_flags: Some(create_string_set([ "-s" ])),
      defines: None
    }),
    (BuildConfigCompilerSpecifier::MSVC, RawBuildConfig {
      compiler_flags: Some(create_string_set([ "/O2" ])),
      link_time_flags: None,
      linker_flags: None,
      defines: None
    })
  ]);

  if include_emscripten_support {
    release_config.insert(
      BuildConfigCompilerSpecifier::Emscripten,
      RawBuildConfig {
        compiler_flags: Some(create_string_set([ "-O3" ])),
        link_time_flags: None,
        linker_flags: None,
        defines: None
      }
    );
  }

  return release_config;
}

fn build_configs_minsizerel_default(include_emscripten_support: bool) -> BuildConfigByCompiler {
  let mut minsizerel_config: BuildConfigByCompiler = BTreeMap::from_iter([
    (BuildConfigCompilerSpecifier::AllCompilers, RawBuildConfig {
      compiler_flags: None,
      link_time_flags: None,
      linker_flags: None,
      defines: Some(create_string_set(["NDEBUG"]))
    }),
    (BuildConfigCompilerSpecifier::GCC, RawBuildConfig {
      compiler_flags: Some(create_string_set([ "-Os" ])),
      link_time_flags: None,
      linker_flags: Some(create_string_set([ "-s" ])),
      defines: None
    }),
    (BuildConfigCompilerSpecifier::Clang, RawBuildConfig {
      compiler_flags: Some(create_string_set([ "-Os" ])),
      link_time_flags: None,
      linker_flags: Some(create_string_set([ "-s" ])),
      defines: None
    }),
    (BuildConfigCompilerSpecifier::MSVC, RawBuildConfig {
      compiler_flags: Some(create_string_set([ "/O1" ])),
      link_time_flags: None,
      linker_flags: None,
      defines: None
    })
  ]);

  if include_emscripten_support {
    minsizerel_config.insert(
      BuildConfigCompilerSpecifier::Emscripten,
      RawBuildConfig {
        compiler_flags: Some(create_string_set([ "-Oz" ])),
        link_time_flags: None,
        linker_flags: None,
        defines: None
      }
    );
  }

  return minsizerel_config;
}

fn build_configs_relwithdebinfo_default(include_emscripten_support: bool) -> BuildConfigByCompiler {
  let mut relwithdebinfo_config: BuildConfigByCompiler = BTreeMap::from_iter([
    (BuildConfigCompilerSpecifier::AllCompilers, RawBuildConfig {
      compiler_flags: None,
      link_time_flags: None,
      linker_flags: None,
      defines: Some(create_string_set(["NDEBUG"]))
    }),
    (BuildConfigCompilerSpecifier::GCC, RawBuildConfig {
      compiler_flags: Some(create_string_set([ "-O2", "-g" ])),
      link_time_flags: None,
      linker_flags: None,
      defines: None
    }),
    (BuildConfigCompilerSpecifier::Clang, RawBuildConfig {
      compiler_flags: Some(create_string_set([ "-O2", "-g" ])),
      link_time_flags: None,
      linker_flags: None,
      defines: None
    }),
    (BuildConfigCompilerSpecifier::MSVC, RawBuildConfig {
      compiler_flags: Some(create_string_set([ "/O2", "/DEBUG" ])),
      link_time_flags: None,
      linker_flags: None,
      defines: None
    })
  ]);

  if include_emscripten_support {
    relwithdebinfo_config.insert(
      BuildConfigCompilerSpecifier::Emscripten,
      RawBuildConfig {
        compiler_flags: Some(create_string_set([ "-O2", "-g", "-gsource-map" ])),
        link_time_flags: None,
        linker_flags: None,
        defines: None
      }
    );
  }

  return relwithdebinfo_config;
}

fn global_defines_default(include_emscripten_support: bool) -> Option<Vec<String>> {
  let mut defines_list: Vec<String> = Vec::new();

  if include_emscripten_support {
    defines_list.push(String::from("((emscripten)) EMSCRIPTEN"));
  }

  return if defines_list.is_empty()
    { None }
    else { Some(defines_list) };
}

pub fn get_default_project_config(
  project_name: &str,
  include_prefix: &str,
  project_lang: &MainFileLanguage,
  project_output_type: &CreationProjectOutputType,
  project_type_creating: &ProjectTypeCreating,
  project_description: &str,
  project_vendor: &str,
  requires_custom_main: Option<bool>
) -> RawProject {
  let include_emscripten_support: bool = should_support_emscripten(project_type_creating);

  RawProject {
    name: project_name.to_string(),
    include_prefix: include_prefix.to_string(),
    description: String::from(project_description),
    vendor: String::from(project_vendor),
    version: String::from("0.0.1"),
    installer_config: None,
    supported_compilers: supported_compilers_default(include_emscripten_support),
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
        windows_icon: None,
        emscripten_html_shell: None,
        link: None,
        build_config: None,
        requires_custom_main
      })
    ]),
    predefined_dependencies: None,
    gcmake_dependencies: None,
    build_configs: BTreeMap::from_iter([
      (BuildType::Debug, build_configs_debug_default(include_emscripten_support)),
      (BuildType::Release, build_configs_release_default(include_emscripten_support)),
      (BuildType::MinSizeRel, build_configs_minsizerel_default(include_emscripten_support)),
      (BuildType::RelWithDebInfo, build_configs_relwithdebinfo_default(include_emscripten_support))
    ]),
    default_build_type: BuildType::Debug,
    global_defines: global_defines_default(include_emscripten_support),
    global_properties: Some(RawGlobalPropertyConfig {
      default_compiled_lib_type: None,
      ipo_enabled_by_default: None
    }),
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

fn create_string_set<'a>(arr: impl IntoIterator<Item=&'a str>) -> Vec<String> {
  return arr
    .into_iter()
    .map(|borrowed_str| String::from(borrowed_str))
    .collect()
}