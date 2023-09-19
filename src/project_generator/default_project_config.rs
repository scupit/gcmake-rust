use std::{collections::{HashMap, BTreeSet, BTreeMap}, iter::FromIterator};

use crate::{project_info::{raw_data_in::{RawProject, RawSubproject, SpecificCompilerSpecifier, RawCompiledItem, OutputItemType, BuildType, BuildConfigCompilerSpecifier, RawBuildConfig, SingleLanguageConfig, LanguageConfigMap, RawTestProject, RawGlobalPropertyConfig, dependencies::user_given_dep_config::UserGivenPredefinedDependencyConfig}}, program_actions::ProjectTypeCreating};

use self::configuration::{MainFileLanguage, OutputLibType, CreationProjectOutputType};

pub mod configuration {
  #[derive(Clone, Copy, PartialEq, Eq)]
  pub enum MainFileLanguage {
    C,
    Cpp,
    Cpp2
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

pub struct DefaultProjectConfigOptions {
  pub supported_compilers: BTreeSet<SpecificCompilerSpecifier>
}

impl DefaultProjectConfigOptions {
  pub fn includes_emscripten_support(&self) -> bool {
    self.supported_compilers.contains(&SpecificCompilerSpecifier::Emscripten)
  }

  pub fn includes_cuda_support(&self) -> bool {
    self.supported_compilers.contains(&SpecificCompilerSpecifier::CUDA)
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

fn supported_compilers(project_type_creating: &ProjectTypeCreating) -> BTreeSet<SpecificCompilerSpecifier> {
  return match project_type_creating {
    ProjectTypeCreating::RootProject { supported_compilers } => BTreeSet::from_iter(supported_compilers.iter().copied()),
    // Compiler support is only used by root projects. Subprojects "inherit" compiler support information
    // in CMake code from the root project.
    _ => BTreeSet::new()
  }
}

type BuildConfigByCompiler = BTreeMap<BuildConfigCompilerSpecifier, RawBuildConfig>;

fn filtered_build_config(
  project_config: &DefaultProjectConfigOptions,
  build_config: BuildConfigByCompiler
) -> BuildConfigByCompiler {
  return build_config.into_iter()
    .filter(|(compiler, _)| match compiler {
      BuildConfigCompilerSpecifier::AllCompilers => true,
      selected => project_config.supported_compilers.contains(&selected.to_specific().unwrap())
    })
    .collect();
}

fn build_configs_debug_default(project_config: &DefaultProjectConfigOptions) -> BuildConfigByCompiler {
  let debug_config: BuildConfigByCompiler = BTreeMap::from_iter([
    (BuildConfigCompilerSpecifier::GCC, RawBuildConfig {
      // https://gcc.gnu.org/onlinedocs/gcc/Optimize-Options.html#index-Og 
      // The GCC docs recommend using -Og for the "standard edit-compile-debug cycle". However,
      // in my experience it causes the debugger to skip lines in places I don't expect.
      // -O0 produces the expected debugging experience every time (so far), so I'm using that
      // as the new default. Unoptimized performance is good enough for debugging most programs
      // anyways. If someone needs -Og for some reason, they can change this in the build config.
      compiler_flags: Some(create_string_set([ "-O0", "-g", "-Wall", "-Wextra", "-Wconversion", "-Wuninitialized", "-pedantic", "-pedantic-errors"])),
      link_time_flags: None,
      linker_flags: None,
      defines: None
    }),
    (BuildConfigCompilerSpecifier::Clang, RawBuildConfig {
      compiler_flags: Some(create_string_set([ "-O0", "-g", "-Wall", "-Wextra", "-Wconversion", "-Wuninitialized", "-pedantic", "-pedantic-errors"])),
      link_time_flags: None,
      linker_flags: None,
      defines: None
    }),
    (BuildConfigCompilerSpecifier::CUDA, RawBuildConfig {
      compiler_flags: None,
      link_time_flags: None,
      linker_flags: None,
      defines: None
    }),
    (BuildConfigCompilerSpecifier::MSVC, RawBuildConfig {
      compiler_flags: Some(create_string_set([ "/Od", "/W4", "/DEBUG", "/RTC1" ])),
      link_time_flags: None,
      linker_flags: None,
      defines: None
    }),
    (BuildConfigCompilerSpecifier::Emscripten, RawBuildConfig {
      compiler_flags: Some(create_string_set([ "-O0", "-g", "-gsource-map" ])),
      link_time_flags: None,
      linker_flags: None,
      defines: None
    })
  ]);

  return filtered_build_config(project_config, debug_config);
}

fn build_configs_release_default(project_config: &DefaultProjectConfigOptions) -> BuildConfigByCompiler {
  let release_config: BuildConfigByCompiler = BTreeMap::from_iter([
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
    (BuildConfigCompilerSpecifier::CUDA, RawBuildConfig {
      compiler_flags: None,
      link_time_flags: None,
      linker_flags: None,
      defines: None
    }),
    (BuildConfigCompilerSpecifier::MSVC, RawBuildConfig {
      compiler_flags: Some(create_string_set([ "/O2" ])),
      link_time_flags: None,
      linker_flags: None,
      defines: None
    }),
    (BuildConfigCompilerSpecifier::Emscripten, RawBuildConfig {
      compiler_flags: Some(create_string_set([ "-O3" ])),
      link_time_flags: None,
      linker_flags: None,
      defines: None
    })
  ]);

  return filtered_build_config(project_config, release_config);
}

fn build_configs_minsizerel_default(project_config: &DefaultProjectConfigOptions) -> BuildConfigByCompiler {
  let minsizerel_config: BuildConfigByCompiler = BTreeMap::from_iter([
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
    (BuildConfigCompilerSpecifier::CUDA, RawBuildConfig {
      compiler_flags: None,
      link_time_flags: None,
      linker_flags: None,
      defines: None
    }),
    (BuildConfigCompilerSpecifier::MSVC, RawBuildConfig {
      compiler_flags: Some(create_string_set([ "/O1" ])),
      link_time_flags: None,
      linker_flags: None,
      defines: None
    }),
    (BuildConfigCompilerSpecifier::Emscripten, RawBuildConfig {
      compiler_flags: Some(create_string_set([ "-Oz" ])),
      link_time_flags: None,
      linker_flags: None,
      defines: None
    })
  ]);

  return filtered_build_config(project_config, minsizerel_config);
}

fn build_configs_relwithdebinfo_default(project_config: &DefaultProjectConfigOptions) -> BuildConfigByCompiler {
  let relwithdebinfo_config: BuildConfigByCompiler = BTreeMap::from_iter([
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
    (BuildConfigCompilerSpecifier::CUDA, RawBuildConfig {
      compiler_flags: None,
      link_time_flags: None,
      linker_flags: None,
      defines: None
    }),
    (BuildConfigCompilerSpecifier::MSVC, RawBuildConfig {
      compiler_flags: Some(create_string_set([ "/O2", "/DEBUG" ])),
      link_time_flags: None,
      linker_flags: None,
      defines: None
    }),
    (BuildConfigCompilerSpecifier::Emscripten, RawBuildConfig {
      compiler_flags: Some(create_string_set([ "-O2", "-g", "-gsource-map" ])),
      link_time_flags: None,
      linker_flags: None,
      defines: None
    })
  ]);

  return filtered_build_config(project_config, relwithdebinfo_config);
}

fn global_defines_default(config: &DefaultProjectConfigOptions) -> Option<Vec<String>> {
  let mut defines_list: Vec<String> = Vec::new();

  if config.includes_emscripten_support() {
    defines_list.push(String::from("((emscripten)) EMSCRIPTEN"));
  }

  if config.includes_cuda_support() {
    defines_list.push(String::from("((cuda)) CUDA"));
  }

  return if defines_list.is_empty()
    { None }
    else { Some(defines_list) };
}

fn needed_predefined_dependencies(
  config: &DefaultProjectConfigOptions,
  requires_cppfront: bool
) -> Option<HashMap<String, UserGivenPredefinedDependencyConfig>> {
  let mut needed_dependencies: HashMap<String, UserGivenPredefinedDependencyConfig> = HashMap::new();

  if requires_cppfront {
    needed_dependencies.insert(
      String::from("cppfront"),
      UserGivenPredefinedDependencyConfig {
        git_tag: Some(String::from("master")),
        commit_hash: None,
        file_version: None,
        repo_url: None,
        options: None
      }
    );
  }

  if config.includes_cuda_support() {
    needed_dependencies.insert(
      String::from("cuda"),
      UserGivenPredefinedDependencyConfig {
        git_tag: None,
        commit_hash: None,
        file_version: None,
        repo_url: None,
        options: None
      }
    );
  }

  return if needed_dependencies.is_empty() {
    None
  }
  else {
    Some(needed_dependencies)
  }
}

fn language_config(
  config: &DefaultProjectConfigOptions,
  requires_cppfront: bool
) -> LanguageConfigMap {
  let mut default_lang_config = LanguageConfigMap {
    c: Some(SingleLanguageConfig {
      // Should this be 99?
      min_standard: String::from("11"),
      exact_standard: None
    }),
    cpp: Some(SingleLanguageConfig {
      min_standard: default_cpp_standard(requires_cppfront).to_string(),
      exact_standard: None
    }),
    cuda: None
  };

  if config.includes_cuda_support() {
    default_lang_config.cuda = Some(SingleLanguageConfig {
      min_standard: String::from("17"),
      exact_standard: None
    });
  }

  return default_lang_config;
}

fn default_cpp_standard(requires_cppfront: bool) -> &'static str {
  return if requires_cppfront
    // To ensure cppfront works properly, we must compiler using c++20.
    // See https://github.com/hsutter/cppfront#how-do-i-build-cppfront
    { "20" }
    else { "17" };
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
  let config_options = DefaultProjectConfigOptions {
    supported_compilers: supported_compilers(project_type_creating)
  };

  let requires_cppfront: bool = match project_lang {
    MainFileLanguage::Cpp2 => true,
    _ => false
  };

  RawProject {
    name: project_name.to_string(),
    include_prefix: include_prefix.to_string(),
    description: String::from(project_description),
    vendor: String::from(project_vendor),
    version: String::from("0.0.1"),
    installer_config: None,
    supported_compilers: config_options.supported_compilers.clone(),
    prebuild_config: None,
    documentation: None,
    features: None,
    languages: language_config(&config_options, requires_cppfront),
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
        defines: None,
        windows_icon: None,
        emscripten_html_shell: None,
        link: None,
        language_features: None,
        build_config: None,
        requires_custom_main
      })
    ]),
    predefined_dependencies: needed_predefined_dependencies(&config_options, requires_cppfront),
    gcmake_dependencies: None,
    build_configs: BTreeMap::from_iter([
      (BuildType::Debug, build_configs_debug_default(&config_options)),
      (BuildType::Release, build_configs_release_default(&config_options)),
      (BuildType::MinSizeRel, build_configs_minsizerel_default(&config_options)),
      (BuildType::RelWithDebInfo, build_configs_relwithdebinfo_default(&config_options))
    ]),
    default_build_type: BuildType::Debug,
    global_defines: global_defines_default(&config_options),
    global_properties: Some(RawGlobalPropertyConfig {
      default_compiled_lib_type: None,
      ipo_enabled_by_default_for: Some(BTreeSet::from([
        BuildType::Release,
        BuildType::MinSizeRel,
        BuildType::RelWithDebInfo
      ])),
      are_language_extensions_enabled: None
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
  let extension: &str;
  let file_name: &str;

  match *project_type {
    CreationProjectOutputType::Executable => {
      file_name = "main";
      extension = match project_lang {
        MainFileLanguage::C => "c",
        MainFileLanguage::Cpp => "cpp",
        MainFileLanguage::Cpp2 => "cpp2"
      };
    },
    CreationProjectOutputType::Library(_) => {
      file_name = project_name;
      extension = match project_lang {
        MainFileLanguage::C => "h",
        MainFileLanguage::Cpp
          | MainFileLanguage::Cpp2 => "hpp",
      };
    }
  };


  return format!("{}.{}", file_name, extension);
}

fn create_string_set<'a>(arr: impl IntoIterator<Item=&'a str>) -> Vec<String> {
  return arr
    .into_iter()
    .map(|borrowed_str| String::from(borrowed_str))
    .collect()
}