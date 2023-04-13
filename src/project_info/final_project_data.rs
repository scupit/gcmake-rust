use std::{collections::{HashMap, HashSet, BTreeMap, BTreeSet}, path::{Path, PathBuf}, io, rc::Rc, fs::{self}, iter::FromIterator};

use crate::{project_info::path_manipulation::cleaned_pathbuf, logger};

use super::{path_manipulation::{cleaned_path_str, relative_to_project_root, absolute_path}, final_dependencies::{FinalGCMakeDependency, FinalPredefinedDependencyConfig}, raw_data_in::{RawProject, dependencies::internal_dep_config::AllRawPredefinedDependencies, BuildType, LanguageConfigMap, OutputItemType, PreBuildConfigIn, SpecificCompilerSpecifier, BuildConfigCompilerSpecifier, TargetSpecificBuildType, LinkSection, RawTestFramework, DefaultCompiledLibType, RawCompiledItem, RawDocumentationGeneratorConfig, RawDocGeneratorName}, final_project_configurables::{FinalProjectType}, CompiledOutputItem, helpers::{parse_subproject_data, parse_root_project_data, populate_existing_files, find_prebuild_script, PrebuildScriptFile, validate_raw_project_outputs, ProjectOutputType, RetrievedCodeFileType, code_file_type, parse_test_project_data, find_doxyfile_in, validate_doxyfile_in, SphinxConfigFiles, find_sphinx_files, validate_conf_py_in}, PreBuildScript, FinalTestFramework, base_include_prefix_for_test, gcmake_constants::{SRC_DIR_NAME, INCLUDE_DIR_NAME, TESTS_DIR_NAME, SUBPROJECTS_DIR_NAME, DOCS_DIR_NAME}, FinalInstallerConfig, CompilerDefine, FinalBuildConfigMap, make_final_build_config_map, FinalTargetBuildConfigMap, FinalGlobalProperties, FinalShortcutConfig, parsers::{version_parser::ThreePartVersion, general_parser::ParseSuccess}, platform_spec_parser::parse_leading_constraint_spec, SystemSpecifierWrapper, FinalFeatureConfig, FinalFeatureEnabler, CodeFileInfo, FileRootGroup, PreBuildScriptType, FinalDocGeneratorName, FinalDocumentationInfo, CodeFileLang, GivenConstraintSpecParseContext};
use colored::*;

const SUBPROJECT_JOIN_STR: &'static str = "_S_";
const TEST_PROJECT_JOIN_STR: &'static str = "_TP_";
const TEST_TARGET_JOIN_STR: &'static str = "_T_";

fn resolve_prebuild_script(
  project_root: &str,
  pre_build_config: &PreBuildConfigIn,
  valid_feature_list: Option<&Vec<&str>>,
  file_root_group: &FileRootGroup
) -> Result<Option<PreBuildScript>, String> {
  let mut generated_file_set: BTreeSet<CodeFileInfo> = BTreeSet::new();

  if let Some(specified_set) = pre_build_config.generated_code.as_ref() {
    let absolute_project_root: PathBuf = absolute_path(&file_root_group.project_root)?;

    for single_generated_file in specified_set {
      let relative_file_root: &Path = match code_file_type(single_generated_file) {
        RetrievedCodeFileType::Source { .. } => file_root_group.src_root.as_path(),
        RetrievedCodeFileType::Header(_)
          | RetrievedCodeFileType::TemplateImpl => file_root_group.header_root.as_path(),
        _ => {
          return Err(format!(
            "Pre-build script specifies generated file \"{}\" which is not a Header, Source, or Template Implementation file. Only code (header, source, template-impl) can be explicitly listed as generated.",
            single_generated_file
          ));
        }
      };

      let file_root: PathBuf = absolute_path(relative_file_root)?;
      let absolute_file_path: PathBuf = absolute_path(file_root.join(single_generated_file))?;

      assert!(
        file_root.starts_with(&absolute_project_root),
        "File root must be inside its project root directory."
      );

      if absolute_file_path.starts_with(&file_root) {
        generated_file_set.insert(
          CodeFileInfo::from_path(
            absolute_file_path.strip_prefix(&absolute_project_root).unwrap(),
            true
          )
        );
      }
      else {
        return Err(format!(
          "Pre-build script attempts to generate file \"{}\" which is outside its root directory \"{}\"." ,
          absolute_file_path.to_str().unwrap(),
          file_root.to_str().unwrap()
        ));
      }
    }
  }

  match find_prebuild_script(project_root) {
    None => return Ok(None),
    Some(script_file) => match script_file {
      PrebuildScriptFile::Exe(entry_file_pathbuf) => {
        let raw_output_item = RawCompiledItem {
          output_type: OutputItemType::Executable,
          requires_custom_main: None,
          emscripten_html_shell: None,
          windows_icon: None,
          defines: None,
          entry_file: relative_to_project_root(project_root, entry_file_pathbuf).to_str().unwrap().to_string(),
          build_config: pre_build_config.build_config.clone(),
          link: pre_build_config.link.clone().map(LinkSection::Uncategorized)
        };

        return Ok(Some(PreBuildScript {
          generated_code: generated_file_set,
          type_config: PreBuildScriptType::Exe(CompiledOutputItem::make_from(
            "Pre-build script",
            &raw_output_item,
            None,
            valid_feature_list
          )?)        
        }));
      },
      PrebuildScriptFile::Python(python_file_pathbuf) => {
        return Ok(Some(PreBuildScript {
          generated_code: generated_file_set,
          type_config: PreBuildScriptType::Python(
            relative_to_project_root(project_root, python_file_pathbuf)
          )
        }))
      }
    }
  }
}

fn feature_list_from(feature_map: &BTreeMap<String, FinalFeatureConfig>) -> Option<Vec<&str>> {
  let feature_name_list: Vec<&str> = feature_map.iter()
    .map(|(feature_name, _)| &feature_name[..])
    .collect();

  return if feature_name_list.is_empty()
    { None }
    else { Some(feature_name_list) }
}

pub struct UseableFinalProjectDataGroup {
  // When determining root project, we don't traverse upward if the project is a GCMake dependency.
  // Therefore it's safe to assume that 'operating_on' and 'root_project' will always be part of the
  // same project tree.
  pub root_project: Rc<FinalProjectData>,
  pub operating_on: Option<Rc<FinalProjectData>>
}

fn project_levels_below_root(clean_path_root: &str) -> io::Result<Option<usize>> {
  let mut levels_up_checked: usize = 0;
  let mut path_using: PathBuf = absolute_path(clean_path_root).unwrap();

  path_using.push("cmake_data.yaml");

  if !path_using.is_file() {
    return Ok(None);
  }

  path_using.pop();

  while path_using.try_exists()? {
    path_using.push("cmake_data.yaml");
    path_using = cleaned_pathbuf(path_using);

    if !path_using.is_file() {
      return Ok(Some(levels_up_checked - 1));
    }

    levels_up_checked += 1;
    path_using.pop();
    path_using.pop();

    // Doesn't traverse up GCMake dependencies. This allows us to assume that the "root project"
    // referenced elsewhere means the project root which contains the specified project directory.
    match path_using.file_name().unwrap().to_str().unwrap() {
      "subprojects" | "tests" => {
        path_using.pop();
      },
      _ => return Ok(Some(levels_up_checked - 1))
    }
  }

  return Ok(None);
}

type SubprojectMap = HashMap<String, Rc<FinalProjectData>>;
type TestProjectMap = SubprojectMap;

pub enum ProjectLoadFailureReason {
  MissingYaml(String),
  MissingRequiredTestFramework(String),
  Other(String)
}

impl ProjectLoadFailureReason {
  pub fn map_message(
    self,
    mapper: impl FnOnce(String) -> String
  ) -> Self {
    match self {
      Self::MissingYaml(err_message) => Self::MissingYaml(mapper(err_message)),
      Self::Other(err_message) => Self::Other(mapper(err_message)),
      Self::MissingRequiredTestFramework(err_message) => Self::MissingRequiredTestFramework(mapper(err_message))
    }
  }

  pub fn extract_message(self) -> String {
    match self {
      Self::MissingYaml(msg) => msg,
      Self::Other(msg) => msg,
      Self::MissingRequiredTestFramework(msg) => msg
    }
  }
}

enum ChildParseMode {
  Subproject,
  TestProject
}

struct NeededParseInfoFromParent {
  actual_base_name: String,
  actual_vendor: String,
  parent_project_namespaced_name: String,
  parse_mode: ChildParseMode,
  test_framework: Option<FinalTestFramework>,
  include_prefix: String,
  target_namespace_prefix: String,
  build_config_map: Rc<FinalBuildConfigMap>,
  language_config_map: Rc<LanguageConfigMap>,
  supported_compilers: Rc<HashSet<SpecificCompilerSpecifier>>,
  inherited_features: Rc<BTreeMap<String, FinalFeatureConfig>>
}

struct CodeFileStats {
  num_cpp2_files: i32,
  num_cpp_files: i32,
  num_c_files: i32
}

impl CodeFileStats {
  pub fn new() -> Self {
    return Self {
      num_cpp2_files: 0,
      num_cpp_files: 0,
      num_c_files: 0
    };
  }

  pub fn requires_cpp(&self) -> bool {
    self.num_cpp2_files + self.num_cpp_files > 0
  }

  pub fn requires_c(&self) -> bool {
    self.num_c_files > 0
  }
}

pub struct FinalProjectLoadContext {
  pub about_to_generate_doxyfile: bool,
  pub about_to_generate_sphinx_files: bool,
  pub just_created_library_project_at: Option<String>
}

impl Default for FinalProjectLoadContext {
  fn default() -> Self {
    Self {
      about_to_generate_doxyfile: false,
      about_to_generate_sphinx_files: false,
      just_created_library_project_at: None
    }
  }
}

#[derive(PartialEq, Eq, Clone, Copy)]
pub enum CppFileGrammar {
  Cpp1,
  Cpp2
}

// NOTE: Link validity is now checked using the DependencyGraph.
pub struct FinalProjectData {
  project_type: FinalProjectType,
  project_output_type: ProjectOutputType,
  project_root_dir: String,
  absolute_project_root: PathBuf,
  pub version: ThreePartVersion,
  // project: RawProject,
  installer_config: FinalInstallerConfig,
  supported_compilers: Rc<HashSet<SpecificCompilerSpecifier>>,
  project_base_name: String,
  full_namespaced_project_name: String,
  project_name_for_error_messages: String,
  description: String,
  vendor: String,
  build_config_map: Rc<FinalBuildConfigMap>,
  default_build_config: BuildType,
  language_config_map: Rc<LanguageConfigMap>,
  global_defines: Vec<CompilerDefine>,
  global_properties: Option<FinalGlobalProperties>,
  documentation: Option<FinalDocumentationInfo>,

  base_include_prefix: String,
  full_include_prefix: String,

  src_dir_relative_to_cwd: String,
  src_dir_relative_to_project_root: String,
  include_dir_relative_to_cwd: String,
  include_dir_relative_to_project_root: String,

  docs_dir_relative_to_cwd: String,
  docs_dir_relative_to_project_root: String,

  // src/FULL_INCLUDE_PREFIX/*.c(pp(2))
  pub src_files: BTreeSet<CodeFileInfo>,
  // src/FULL_INCLUDE_PREFIX/*.private.h(pp)
  pub private_headers: BTreeSet<CodeFileInfo>,
  // include/FULL_INCLUDE_PREFIX/*.h(pp)
  pub include_files: BTreeSet<CodeFileInfo>,
  // include/FULL_INCLUDE_PREFIX/*.{inl|tpp}
  pub template_impl_files: BTreeSet<CodeFileInfo>,

  subprojects: SubprojectMap,
  test_framework: Option<FinalTestFramework>,
  tests: TestProjectMap,
  output: HashMap<String, CompiledOutputItem>,

  predefined_dependencies: HashMap<String, Rc<FinalPredefinedDependencyConfig>>,
  gcmake_dependency_projects: HashMap<String, Rc<FinalGCMakeDependency>>,

  features: Rc<BTreeMap<String, FinalFeatureConfig>>,
  prebuild_script: Option<PreBuildScript>,
  target_namespace_prefix: String,
  was_just_created: bool
}

impl FinalProjectData {
  pub fn new(
    unclean_given_root: &str,
    dep_config: &AllRawPredefinedDependencies,
    project_load_context: FinalProjectLoadContext
  ) -> Result<UseableFinalProjectDataGroup, ProjectLoadFailureReason> {
    let cleaned_given_root: String = cleaned_path_str(unclean_given_root);

    let levels_below_root: usize = match project_levels_below_root(cleaned_given_root.as_str()) {
      Err(err) => return Err(ProjectLoadFailureReason::Other(
        format!("Error when trying to find project level: {}", err.to_string())
      )),
      Ok(maybe_level) => match maybe_level {
        Some(value) => value,
        None => return Err(ProjectLoadFailureReason::MissingYaml(format!(
          "The directory \"{}\" does not contain a {} file, so the project level could not be determined.",
          &cleaned_given_root.yellow(),
          "cmake_data.yaml".yellow()
        )))
      }
    };

    let mut real_project_root_using: PathBuf = PathBuf::from(&cleaned_given_root);

    if levels_below_root > 0 {
      // Current project is <level> levels deep. Need to go back <level> * 2 dirs, since subprojects
      // are nested in the 'subprojects/<subproject name>' directory
      for _ in 0..(levels_below_root * 2) {
        real_project_root_using.push("..");
      }
    }

    let root_project: Rc<FinalProjectData> = Rc::new(Self::create_new(
      real_project_root_using.to_str().unwrap(),
      None,
      dep_config,
      &project_load_context.just_created_library_project_at
        .clone()
        .map(|creation_root| absolute_path(creation_root).unwrap())
    )?);

    root_project.validate_correctness(&project_load_context)
      .map_err(ProjectLoadFailureReason::Other)?;

    return Ok(UseableFinalProjectDataGroup {
      operating_on: Self::find_with_root(
        &absolute_path(cleaned_given_root)
          .map_err(ProjectLoadFailureReason::Other)?,
        Rc::clone(&root_project)
      ),
      root_project,
    });
  }

  fn create_new(
    unclean_project_root: &str,
    parent_project_info: Option<NeededParseInfoFromParent>,
    all_dep_config: &AllRawPredefinedDependencies,
    just_created_project_at: &Option<PathBuf>
  ) -> Result<FinalProjectData, ProjectLoadFailureReason> {
    // NOTE: Subprojects are still considered whole projects, however they are not allowed to specify
    // top level build configuration data. This means that language data, build configs, etc. are not
    // defined in subprojects, and shouldn't be written. Build configuration related data is inherited
    // from the parent project.
    let mut raw_project: RawProject;
    let project_type: FinalProjectType;
    let full_namespaced_project_name: String;

    let final_test_framework: Option<FinalTestFramework>;

    let language_config: Rc<LanguageConfigMap>;
    let build_config: Rc<FinalBuildConfigMap>;
    let supported_compiler_set: Rc<HashSet<SpecificCompilerSpecifier>>;
    let features: Rc<BTreeMap<String, FinalFeatureConfig>>;

    match &parent_project_info {
      None => {
        raw_project = parse_root_project_data(&unclean_project_root)?;
        language_config = Rc::new(raw_project.languages.clone());
        supported_compiler_set = Rc::new(HashSet::from_iter(raw_project.supported_compilers.clone()));
        full_namespaced_project_name = raw_project.name.clone();
        project_type = FinalProjectType::Root;
        features = Rc::new(
          raw_project.features.clone()
            .map_or(Ok(BTreeMap::new()), |feature_map|
              feature_map
                .into_iter()
                .map(|(feature_name, raw_feature)|
                  FinalFeatureConfig::make_from(raw_feature)
                    .map(|final_feature|
                      (
                        feature_name,
                        final_feature
                      )
                    )
                )
                .collect()
            )
            .map_err(ProjectLoadFailureReason::Other)?
        );

        let valid_feature_list: Option<Vec<&str>> = feature_list_from(&features);

        build_config = Rc::new(
          make_final_build_config_map(
            &raw_project.build_configs,
            valid_feature_list.as_ref()
          )
            .map_err(ProjectLoadFailureReason::Other)?,
        );
        final_test_framework = match &raw_project.test_framework {
          None => None,
          Some(raw_framework_info) => {
            // REFACTOR: Pretty sure I can refactor this somehow.
            let test_framework_lib: Rc<FinalPredefinedDependencyConfig> = FinalPredefinedDependencyConfig::new(
              all_dep_config,
              raw_framework_info.lib_config(),
              raw_framework_info.name(),
              valid_feature_list.as_ref()
            )
              .map(|config| Rc::new(config))
              .map_err(ProjectLoadFailureReason::Other)?;
            
            match raw_framework_info {
              RawTestFramework::Catch2(_) => Some(FinalTestFramework::Catch2(test_framework_lib)),
              RawTestFramework::DocTest(_) => Some(FinalTestFramework::DocTest(test_framework_lib)),
              RawTestFramework::GoogleTest(_) => Some(FinalTestFramework::GoogleTest(test_framework_lib))
            }
          }
        };
      }
      Some(NeededParseInfoFromParent {
        parse_mode: ChildParseMode::TestProject,
        test_framework,
        parent_project_namespaced_name,
        supported_compilers,
        build_config_map,
        language_config_map,
        actual_base_name,
        actual_vendor,
        include_prefix: _,
        target_namespace_prefix: _,
        inherited_features
      }) => {
        language_config = Rc::clone(language_config_map);
        supported_compiler_set = Rc::clone(supported_compilers);
        build_config = Rc::clone(build_config_map);
        features = Rc::clone(inherited_features);

        let project_path = PathBuf::from(cleaned_path_str(unclean_project_root));
        let test_project_name: &str = project_path
          .file_name()
          .unwrap()
          .to_str()
          .unwrap();

        raw_project = parse_test_project_data(unclean_project_root)?
          .into_raw_subproject()
          .into();

        raw_project.name = actual_base_name.clone();
        raw_project.vendor = actual_vendor.clone();

        full_namespaced_project_name = format!(
          "{}{}{}",
          parent_project_namespaced_name,
          TEST_PROJECT_JOIN_STR,
          raw_project.get_name()
        );

        match test_framework {
          None => return Err(ProjectLoadFailureReason::MissingRequiredTestFramework(format!(
            "Tried to configure test project '{}' (path: '{}'), however the toplevel project did not specify a test framework. To enable testing, specify a test_framework in the toplevel project.",
            test_project_name,
            cleaned_path_str(unclean_project_root)
          ))),
          Some(framework) => {
            project_type = FinalProjectType::Test {
              framework: framework.clone()
            };
          }
        }
        final_test_framework = test_framework.clone();
      },
      Some(NeededParseInfoFromParent {
        parse_mode: ChildParseMode::Subproject,
        test_framework,
        parent_project_namespaced_name,
        language_config_map,
        supported_compilers,
        build_config_map,
        actual_base_name,
        actual_vendor,
        include_prefix: _,
        target_namespace_prefix: _,
        inherited_features
      }) => {
        language_config = Rc::clone(language_config_map);
        supported_compiler_set = Rc::clone(supported_compilers);
        build_config = Rc::clone(build_config_map);
        features = Rc::clone(inherited_features);

        raw_project = parse_subproject_data(&unclean_project_root)?.into();
        raw_project.name = actual_base_name.clone();
        raw_project.vendor = actual_vendor.clone();

        full_namespaced_project_name = format!(
          "{}{}{}",
          parent_project_namespaced_name,
          SUBPROJECT_JOIN_STR,
          raw_project.get_name()
        );
        project_type = FinalProjectType::Subproject { };
        final_test_framework = test_framework.clone();
      }
    }

    let valid_feature_list: Option<Vec<&str>> = if features.is_empty() {
      None
    }
    else {
      Some(
        features.keys()
          .map(|key| &key[..])
          .collect()
      )
    };

    let project_output_type: ProjectOutputType = match validate_raw_project_outputs(&raw_project) {
      Ok(project_output_type) => project_output_type,
      Err(err_message) => return Err(ProjectLoadFailureReason::Other(err_message))
    };

    let full_include_prefix: String;
    let target_namespace_prefix: String;

    match parent_project_info {
      Some(parent_project) => {
        let true_base_prefix: String = match &parent_project.parse_mode {
          ChildParseMode::TestProject => base_include_prefix_for_test(raw_project.get_include_prefix()),
          _ => raw_project.get_include_prefix().to_string()
        };

        full_include_prefix = format!(
          "{}/{}",
          parent_project.include_prefix,
          true_base_prefix
        );

        target_namespace_prefix = parent_project.target_namespace_prefix;
      },
      None => {
        full_include_prefix = raw_project.get_include_prefix().to_string();
        target_namespace_prefix = raw_project.get_name().to_string();
      }
    }

    let project_root_relative_to_cwd: String = cleaned_path_str(&unclean_project_root).to_string();
    let project_vendor: String = raw_project.vendor.clone();

    let src_dir_relative_to_project_root: String = format!(
      "{}/{}",
      SRC_DIR_NAME,
      &full_include_prefix
    );

    let src_dir_relative_to_cwd: String = format!(
      "{}/{}",
      &project_root_relative_to_cwd,
      &src_dir_relative_to_project_root
    );

    let include_dir_relative_to_project_root: String = format!(
      "{}/{}",
      INCLUDE_DIR_NAME,
      &full_include_prefix
    );

    let include_dir_relative_to_cwd: String = format!(
      "{}/{}",
      &project_root_relative_to_cwd,
      &include_dir_relative_to_project_root
    );

    let docs_dir_relative_to_project_root: String = String::from(DOCS_DIR_NAME);

    let docs_dir_relative_to_cwd: String = format!(
      "{}/{}",
      &project_root_relative_to_cwd,
      &docs_dir_relative_to_project_root
    );

    let mut test_project_map: SubprojectMap = SubprojectMap::new();

    let project_test_dir_path: PathBuf = PathBuf::from(format!(
      "{}/{}",
      &project_root_relative_to_cwd,
      TESTS_DIR_NAME
    ));

    if project_test_dir_path.is_dir() {
      let tests_dir_iter = fs::read_dir(project_test_dir_path.as_path())
        .map_err(|err| ProjectLoadFailureReason::Other(err.to_string()))?;

      for dir_entry in tests_dir_iter {
        let test_project_path: PathBuf = match dir_entry {
          Ok(entry) => entry.path(),
          Err(err) => return Err(ProjectLoadFailureReason::Other(err.to_string()))
        };
      
        if test_project_path.is_dir() {
          let test_project_name: String = test_project_path.file_name().unwrap().to_str().unwrap().to_string();

          let new_test_project: FinalProjectData = Self::create_new(
            test_project_path.to_str().unwrap(),
            Some(NeededParseInfoFromParent {
              actual_base_name: test_project_name.clone(),
              actual_vendor: project_vendor.clone(),
              parent_project_namespaced_name: full_namespaced_project_name.clone(),
              parse_mode: ChildParseMode::TestProject,
              test_framework: final_test_framework.clone(), 
              include_prefix: full_include_prefix.clone(),
              target_namespace_prefix: target_namespace_prefix.clone(),
              build_config_map: Rc::clone(&build_config),
              language_config_map: Rc::clone(&language_config),
              supported_compilers: Rc::clone(&supported_compiler_set),
              inherited_features: Rc::clone(&features)
            }),
            all_dep_config,
            just_created_project_at
          )
            .map_err(|failure_reason| {
              failure_reason.map_message(|err_message| format!(
                "\t-> in test project '{}'\n{}",
                cleaned_pathbuf(test_project_path.clone()).to_str().unwrap(),
                err_message
              ))
            })?;

          test_project_map.insert(
            test_project_name,
            Rc::new(new_test_project)
          );
        }
      }
    }

    let project_subproject_dir_path: PathBuf = PathBuf::from(format!(
      "{}/{}",
      &project_root_relative_to_cwd,
      SUBPROJECTS_DIR_NAME
    ));

    let mut subprojects: SubprojectMap = SubprojectMap::new();

    if project_subproject_dir_path.is_dir() {
      let subprojects_dir_iter = fs::read_dir(project_subproject_dir_path.as_path())
        .map_err(|err| ProjectLoadFailureReason::Other(err.to_string()))?;

      for dir_entry in subprojects_dir_iter {
        let subproject_path: PathBuf = match dir_entry {
          Ok(entry) => entry.path(),
          Err(err) => return Err(ProjectLoadFailureReason::Other(err.to_string()))
        };
      
        if subproject_path.is_dir() {
          let subproject_name: String = subproject_path.file_name().unwrap().to_str().unwrap().to_string();

          let new_subproject: FinalProjectData = Self::create_new(
            subproject_path.to_str().unwrap(),
            Some(NeededParseInfoFromParent {
              actual_base_name: subproject_name.clone(),
              actual_vendor: project_vendor.clone(),
              parent_project_namespaced_name: full_namespaced_project_name.clone(),
              parse_mode: ChildParseMode::Subproject,
              test_framework: final_test_framework.clone(),
              include_prefix: full_include_prefix.clone(),
              target_namespace_prefix: target_namespace_prefix.clone(),
              supported_compilers: Rc::clone(&supported_compiler_set),
              build_config_map: Rc::clone(&build_config),
              language_config_map: Rc::clone(&language_config),
              inherited_features: Rc::clone(&features)
            }),
            all_dep_config,
            just_created_project_at
          )
            .map_err(|failure_reason| {
              failure_reason.map_message(|err_message| format!(
                "\t-> in subproject '{}'\n{}",
                cleaned_pathbuf(subproject_path.clone()).to_str().unwrap(),
                err_message
              ))
            })?;

          subprojects.insert(
            subproject_name,
            Rc::new(new_subproject)
          );
        }
      }
    }

    let mut gcmake_dependency_projects: HashMap<String, Rc<FinalGCMakeDependency>> = HashMap::new();

    if let Some(gcmake_dep_map) = &raw_project.gcmake_dependencies {
      for (dep_name, dep_config) in gcmake_dep_map {
        let dep_path: String = format!("{}/dep/{}", &project_root_relative_to_cwd, &dep_name);

        let maybe_dep_project: Option<Rc<FinalProjectData>> = if Path::new(&dep_path).exists() {
          Some(Rc::new(Self::create_new(
            &dep_path,
            None,
            all_dep_config,
            just_created_project_at
          )?))
        }
        else { None };

        gcmake_dependency_projects.insert(
          dep_name.clone(),
          Rc::new(
            FinalGCMakeDependency::new(
              &dep_name,
              dep_config,
              maybe_dep_project
            )
            .map_err(ProjectLoadFailureReason::Other)?
          )
        );
      }
    }

    let mut output_items: HashMap<String, CompiledOutputItem> = HashMap::new();

    for (output_name, raw_output_item) in raw_project.get_output_mut() {
      if let FinalProjectType::Test { framework } = &project_type {
        if raw_output_item.link.is_none() {
          raw_output_item.link = Some(LinkSection::Uncategorized(Vec::new()));
        }

        let needed_target_name: &str = if raw_output_item.requires_custom_main.unwrap_or(false)
          { framework.main_not_provided_link_target_name() }
          else { framework.main_provided_link_target_name() };

        raw_output_item.link.as_mut().unwrap().add_exe_link(
          framework.project_dependency_name(),
          needed_target_name
        );
      }

      let actual_output_name: &str;
      let system_spec: Option<SystemSpecifierWrapper>;

      {
        let parsing_context = GivenConstraintSpecParseContext {
          maybe_valid_feature_list: valid_feature_list.as_ref(),
          is_before_output_name: true
        };

        // TODO: Disallow usage of language feature constraints for output items themselves. 
        match parse_leading_constraint_spec(output_name, parsing_context) {
          Ok(Some(ParseSuccess { value: system_spec_wrapper, rest: real_output_name })) => {
            actual_output_name = real_output_name;
            system_spec = Some(system_spec_wrapper);
          },
          Ok(None) => {
            actual_output_name = output_name;
            system_spec = None;
          },
          Err(err_msg) => return Err(ProjectLoadFailureReason::Other(
            format!("Error when parsing system specifier from output name '{}':\n{}", output_name, err_msg)
          ))
        }

      }

      output_items.insert(
        actual_output_name.to_string(),
        CompiledOutputItem::make_from(
          actual_output_name,
          raw_output_item,
          system_spec,
          valid_feature_list.as_ref()
        )
          .map_err(|err_message| ProjectLoadFailureReason::Other(
            format!("When creating output item named '{}':\n{}", output_name, err_message)
          ))?
      );
    }

    let mut predefined_dependencies: HashMap<String, Rc<FinalPredefinedDependencyConfig>> = HashMap::new();

    if let FinalProjectType::Root = &project_type {
      if let Some(framework) = &final_test_framework {
        predefined_dependencies.insert(
          framework.project_dependency_name().to_string(),
          framework.unwrap_config()
        );
      }
    }

    if let Some(pre_deps) = &raw_project.predefined_dependencies {
      for (dep_name, user_given_config) in pre_deps {
        let finalized_dep = FinalPredefinedDependencyConfig::new(
          all_dep_config,
          user_given_config,
          dep_name,
          valid_feature_list.as_ref()
        )
          .map_err(ProjectLoadFailureReason::Other)?;

        predefined_dependencies.insert(dep_name.clone(), Rc::new(finalized_dep));
      }
    }

    let file_root_group = FileRootGroup {
      project_root: PathBuf::from(&project_root_relative_to_cwd),
      header_root: PathBuf::from(&include_dir_relative_to_cwd),
      src_root: PathBuf::from(&src_dir_relative_to_cwd)
    };

    let prebuild_script = resolve_prebuild_script(
      &project_root_relative_to_cwd,
      raw_project.prebuild_config.as_ref().unwrap_or(&PreBuildConfigIn {
        link: None,
        build_config: None,
        generated_code: None
      }),
      valid_feature_list.as_ref(),
      &file_root_group
    ).map_err(ProjectLoadFailureReason::Other)?;

    let maybe_version: Option<ThreePartVersion> = ThreePartVersion::from_str(raw_project.get_version());

    if maybe_version.is_none() {
      return Err(ProjectLoadFailureReason::Other(format!(
        "Invalid project version '{}' given. Version must be formatted like a normal three-part version (ex: 1.0.0), and may be prefixed with the letter 'v'.",
        raw_project.get_version()
      )));
    }

    let installer_config: FinalInstallerConfig = match &raw_project.installer_config {
      None => FinalInstallerConfig {
        title: raw_project.name.clone(),
        description: raw_project.description.clone(),
        name_prefix: raw_project.name.clone(),
        shortcuts: HashMap::new()
      },
      Some(raw_inst_config) => FinalInstallerConfig {
        title: raw_inst_config.title.clone().unwrap_or(raw_project.name.clone()),
        description: raw_inst_config.description.clone().unwrap_or(raw_project.description.clone()),
        name_prefix: raw_inst_config.name_prefix.clone().unwrap_or(raw_project.name.clone()),
        shortcuts: raw_inst_config.shortcuts.clone()
          .unwrap_or(HashMap::new())
          .into_iter()
          .map(|(target_name, raw_shortcut_config)|
            (target_name, FinalShortcutConfig::from(raw_shortcut_config))
          )
          .collect()
      }
    };

    let project_name_for_error_messages: String = full_namespaced_project_name
      .split(SUBPROJECT_JOIN_STR)
      .collect::<Vec<&str>>()
      .join(" => ")
      .split(TEST_PROJECT_JOIN_STR)
      .collect::<Vec<&str>>()
      .join(" -> ");

    let global_defines: Vec<CompilerDefine> = raw_project.global_defines
      .as_ref()
      .map_or(
        Ok(Vec::new()),
        |defines_set| CompilerDefine::make_list(&defines_set, valid_feature_list.as_ref())
      )
      .map_err(ProjectLoadFailureReason::Other)?;

    let mut finalized_project_data = FinalProjectData {
      project_base_name: raw_project.name.clone(),
      project_name_for_error_messages,
      full_namespaced_project_name,
      description: raw_project.description.to_string(),
      version: maybe_version.unwrap(),
      installer_config,
      vendor: project_vendor,
      full_include_prefix,
      base_include_prefix: raw_project.get_include_prefix().to_string(),
      global_defines: global_defines,
      documentation: Self::finalized_doc_generator_info(raw_project.documentation.as_ref()),
      docs_dir_relative_to_cwd,
      docs_dir_relative_to_project_root,
      features,
      global_properties: raw_project.global_properties
        .as_ref()
        .map(FinalGlobalProperties::from_raw),
      build_config_map: build_config,
      default_build_config: raw_project.default_build_type,
      language_config_map: language_config,
      supported_compilers: supported_compiler_set,
      project_type,
      project_output_type,
      absolute_project_root: absolute_path(&project_root_relative_to_cwd)
        .map_err(ProjectLoadFailureReason::Other)?,
      project_root_dir: project_root_relative_to_cwd,

      src_dir_relative_to_cwd,
      src_dir_relative_to_project_root,
      include_dir_relative_to_cwd,
      include_dir_relative_to_project_root,

      src_files: BTreeSet::new(),
      private_headers: BTreeSet::new(),
      include_files: BTreeSet::new(),
      template_impl_files: BTreeSet::new(),
      subprojects,
      output: output_items,
      predefined_dependencies,
      gcmake_dependency_projects,
      prebuild_script,
      target_namespace_prefix,
      test_framework: final_test_framework,
      tests: test_project_map,
      was_just_created: false
    };

    finalized_project_data.was_just_created = match just_created_project_at {
      Some(created_root) => *created_root == finalized_project_data.absolute_project_root,
      None => false
    };

    if let Some(pre_build) = &finalized_project_data.prebuild_script {
      for generated_code_file in &pre_build.generated_code {
        let cloned_file_info: CodeFileInfo = generated_code_file.clone();

        // NOTE: All generated files will already be listed as part of the project's files.
        // This is fine because is_generated == true for each of these files, so we can tell
        // that they might not exist on the file system already.
        match generated_code_file.code_file_type() {
          RetrievedCodeFileType::Source { .. } => {
            finalized_project_data.src_files.insert(cloned_file_info);
          },
          RetrievedCodeFileType::Header(_) | RetrievedCodeFileType::TemplateImpl => {
            finalized_project_data.include_files.insert(cloned_file_info);
          },
          _ => ()
        }
      }
    }

    let usable_project_root = PathBuf::from(finalized_project_data.get_project_root_dir());

    populate_existing_files(
      usable_project_root.as_path(),
      Path::new(&finalized_project_data.src_dir_relative_to_cwd),
      &mut finalized_project_data.src_files,
      &|file_path| match code_file_type(file_path) {
        RetrievedCodeFileType::Source { .. } => true,
        _ => false
      }
    )
      .map_err(|err| ProjectLoadFailureReason::Other(err.to_string()))?;

    populate_existing_files(
      usable_project_root.as_path(),
      Path::new(&finalized_project_data.src_dir_relative_to_cwd),
      &mut finalized_project_data.private_headers,
      &|file_path| match code_file_type(file_path) {
        RetrievedCodeFileType::Header(_)
          | RetrievedCodeFileType::TemplateImpl => true,
        _ => false
      }
    )
      .map_err(|err| ProjectLoadFailureReason::Other(err.to_string()))?;

    populate_existing_files(
      usable_project_root.as_path(),
      Path::new(&finalized_project_data.include_dir_relative_to_cwd),
      &mut finalized_project_data.include_files,
      &|file_path| match code_file_type(file_path) {
        RetrievedCodeFileType::Header(_) => true,
        _ => false
      }
    )
      .map_err(|err| ProjectLoadFailureReason::Other(err.to_string()))?;

    populate_existing_files(
      usable_project_root.as_path(),
      Path::new(&finalized_project_data.include_dir_relative_to_cwd),
      &mut finalized_project_data.template_impl_files,
      &|file_path| match code_file_type(file_path) {
        RetrievedCodeFileType::TemplateImpl => true,
        _ => false
      }
    )
      .map_err(|err| ProjectLoadFailureReason::Other(err.to_string()))?;

    return Ok(finalized_project_data);
  }

  pub fn is_test_project(&self) -> bool {
    match &self.project_type {
      FinalProjectType::Test { .. } => true,
      _ => false
    }
  }

  pub fn is_root_project(&self) -> bool {
    match &self.project_type {
      FinalProjectType::Root => true,
      _ => false
    }
  }

  // Visit the toplevel root project and all its subprojects.
  fn find_with_root(
    absolute_root: &PathBuf,
    project: Rc<FinalProjectData>
  ) -> Option<Rc<FinalProjectData>> {
    if project.absolute_project_root == *absolute_root {
      return Some(project);
    }

    for (_, subproject) in &project.subprojects {
      if let Some(matching_project) = Self::find_with_root(absolute_root, Rc::clone(subproject)) {
        return Some(matching_project);
      }
    }

    for (_, test_project) in &project.tests {
      if let Some(matching_project) = Self::find_with_root(absolute_root, Rc::clone(test_project)) {
        return Some(matching_project);
      }
    }

    None
  }

  // TODO: Return an iterator instead of using a callback.
  fn iter_all_code_files(&self, mut callback: &mut dyn FnMut(&CodeFileInfo)) {
    let file_sets: Vec<&BTreeSet<CodeFileInfo>> = vec![
      &self.private_headers,
      &self.include_files,
      &self.src_files,
      &self.template_impl_files
    ];

    for file_set in file_sets {
      for code_file in file_set {
        callback(code_file);
      }
    }

    if let Some(pre_build_config) = self.prebuild_script.as_ref() {
      // NOTE: We can ignore generated_code files listed in the pre-build script because those
      // are appended to the project's files.
      if let PreBuildScriptType::Exe(exe_pre_build) = pre_build_config.get_type() {
        callback(exe_pre_build.get_entry_file());
      }
    }

    for (_, target_config) in self.get_outputs() {
      callback(target_config.get_entry_file());
    }

    for (_, subproject_config) in self.get_subprojects() {
      subproject_config.iter_all_code_files(&mut callback);
    }

    for (_, test_project_config) in self.get_test_projects() {
      test_project_config.iter_all_code_files(&mut callback);
    }
  }

  fn get_code_file_stats(&self) -> CodeFileStats {
    let mut file_stats = CodeFileStats::new();

    self.iter_all_code_files(&mut |code_file| {
      match code_file.language().unwrap() {
        CodeFileLang::C => file_stats.num_c_files += 1,
        CodeFileLang::Cpp { used_grammar } => match used_grammar {
          CppFileGrammar::Cpp1 => file_stats.num_cpp_files += 1,
          CppFileGrammar::Cpp2 => file_stats.num_cpp2_files += 1
        }
      }
    });

    return file_stats;
  }
 
  fn ensure_language_config_correctness(&self) -> Result<(), String> {
    let language_config = self.get_language_info();
    let code_file_stats: CodeFileStats = self.get_code_file_stats();
    
    let project_uses_cpp: bool = code_file_stats.requires_cpp();
    let project_uses_c: bool = code_file_stats.requires_c();

    if project_uses_c {
      // Maybe make this a global if needed. It may be useful to print a list of supported language
      // standard "versions".
      let allowed_c_standards = [90, 99, 11, 17, 23];

      match language_config.c.as_ref() {
        None => return Err(format!(
          "Project [{}] makes use of C, but has not specified any C language configuration. Fix by adding a C language configuration to cmake_data.yaml:\n\n{}:\n  {}:\n    standard: 11",
          self.get_name_for_error_messages().yellow(),
          "languages".purple(),
          "c".green()
        )),
        Some(c_config) => {
          if !allowed_c_standards.contains(&c_config.standard) {
            return Err(format!(
              "C Language standard must be one of [90, 99, 11, 17, 23], but {} was given",
              c_config.standard.to_string().red()
            ));
          }
        }
      }
    }

    if project_uses_cpp {
      let allowed_cpp_standards = [98, 11, 14, 17, 20, 23];

      match language_config.cpp.as_ref() {
        None => return Err(format!(
          "Project [{}] makes use of C++, but has not specified any C++ language configuration. Fix by adding a C++ language configuration to cmake_data.yaml:\n\n{}:\n  {}:\n    standard: 17",
          self.get_name_for_error_messages().yellow(),
          "languages".purple(),
          "cpp".green()
        )),
        Some(cpp_config) => {
          if !allowed_cpp_standards.contains(&cpp_config.standard) {
            return Err(format!(
              "C++ Language standard must be one of [98, 11, 14, 17, 20, 23], but {} was given",
              cpp_config.standard.to_string().red()
            ));
          }

          if self.any_files_contain_cpp2_grammar() && ![20, 23].contains(&cpp_config.standard) {
            logger::block(|| {
              logger::warn(format!(
                "Project [{}] contains .cpp2 files, but its C++ standard is currently set to {}. cppfront (.cpp2) requires C++20 or higher. Please set the Cpp language standard to {} or {} in cmake_data.yaml. Example:\n",
                self.get_name_for_error_messages().yellow(),
                cpp_config.standard.to_string().red(),
                "20".green(),
                "23".green()
              ));

              println!(
                "languages:\n  cpp:\n    standard: {}",
                "20".green()
              );
            })
          }
        }
      }
    }

    Ok(())
  }

  fn finalized_doc_generator_info(info: Option<&RawDocumentationGeneratorConfig>) -> Option<FinalDocumentationInfo> {
    match info {
      None => None,
      Some(raw_doc_generator_info) => {
        let generator: FinalDocGeneratorName = match &raw_doc_generator_info.generator {
          RawDocGeneratorName::Doxygen => FinalDocGeneratorName::Doxygen,
          RawDocGeneratorName::Sphinx => FinalDocGeneratorName::Sphinx
        };

        return Some(FinalDocumentationInfo {
          generator,
          headers_only: raw_doc_generator_info.headers_only.unwrap_or(true),
          include_private_headers: raw_doc_generator_info.include_private_headers.unwrap_or(false)
        });
      }
    }
  }

  fn ensure_build_config_correctness(&self) -> Result<(), String> {
    if let Some(global_props) = self.global_properties.as_ref() {
      for build_type in &global_props.ipo_enabled_by_default_for {
        if !self.get_build_configs().contains_key(build_type) {
          let build_type_name: &str = build_type.name_str();

          return Err(format!(
            "Config issue: Global property '{}' tries to enable IPO by default for the '{}' build configuration, but the project doesn't have a '{}' configuration in its build_configs. To fix, either add a '{}' configuration to build_configs in cmake_data.yaml or remove it from the 'ipo_enabled_by_default_for' list.",
            "ipo_enabled_by_default_for".red(),
            build_type_name.yellow(),
            build_type_name.yellow(),
            build_type_name.yellow()
          ));
        }
      }
    }

    for (build_type, by_compiler_map) in self.get_build_configs() {
      for (config_compiler, _) in by_compiler_map {
        if let Some(specific_compiler) = config_compiler.to_specific() {
          if !self.supported_compilers.contains(&specific_compiler) {
            let compiler_name: &str = specific_compiler.name_string();

            return Err(format!(
              "Config Issue: '{}' build config defines a section for {}, but {} is not in the supported_compilers list. To fix, either remove the {} section or add {} to the supported_compilers list for this project.",
              build_type.name_str(),
              compiler_name,
              compiler_name,
              compiler_name,
              compiler_name
            ));
          }
        }
      }
    }

    Ok(())
  }

  fn validate_correctness(&self, project_load_context: &FinalProjectLoadContext) -> Result<(), String> {
    if self.get_project_base_name().contains(' ') {
      return Err(format!(
        "Project name cannot contain spaces, but does. (Currently: {})",
        self.get_project_base_name()
      ));
    }

    if self.get_full_include_prefix().contains(' ') {
      return Err(format!(
        "Project 'include prefix' cannot contain spaces, but does. (Currently: {})",
        self.get_full_include_prefix()
      ));
    }

    if self.supported_compilers.contains(&SpecificCompilerSpecifier::Emscripten) && !self.supports_emscripten() {
      return Err(format!(
        "Emscripten is listed as a supported compiler, but the project's contains dependencies which do not support compilation with Emscripten."
      ))
    }

    self.validate_features()?;
    self.validate_header_names()?;
    self.ensure_doc_generator_correctness(project_load_context)?;
    self.ensure_no_file_collision()?;

    for (_, test_project) in &self.tests {
      if let ProjectOutputType::ExeProject = &test_project.project_output_type {
        test_project.validate_correctness(&project_load_context)?;
      }
      else {
        return Err(format!(
          "Test project '{}' in '{}' is not an executable project. All tests must output only executables.",
          test_project.get_project_base_name(),
          self.get_project_base_name()
        ));
      }
    }

    for (_, subproject) in &self.subprojects {
      subproject.validate_correctness(&project_load_context)?;
    }

    self.ensure_language_config_correctness()?;
    self.ensure_build_config_correctness()?;
    self.validate_project_type_specific_info()?;

    for (output_name, output_item) in &self.output {
      let the_item_name: String = format!("output \"{}\"", output_name);
      self.validate_target_info(&the_item_name, output_item, false)?;
    }

    if let Some(existing_script) = &self.prebuild_script {
      if let PreBuildScriptType::Exe(script_exe_config) = existing_script.get_type() {
        let the_item_name: String = format!("pre-build script (for project [{}])", self.get_name_for_error_messages());
        self.validate_target_info(&the_item_name, script_exe_config, true)?;
      }
    }

    self.validate_installer_config()?;

    Ok(())
  }

  fn warn_unused_doc_config_file(
    &self,
    doc_generator: FinalDocGeneratorName,
    file_relative_to_cwd: impl AsRef<Path>
  ) {
    let config_tool_name: &str = doc_generator.to_str();

    logger::warn(format!(
      "Project [{}] contains file {}, but hasn't enabled a documentation generator in its cmake_data.yaml. If this is intended, just ignore this warning. Otherwise, enable the {} documentation generator in cmake_data.yaml like this:\n\n{}:\n   generator: {}",
      self.get_name_for_error_messages().yellow(),
      file_relative_to_cwd.as_ref().to_str().unwrap(),
      config_tool_name.green(),
      "documentation".purple(),
      config_tool_name.green()
    ));
  }

  fn err_for_missing_doc_config_file(
    &self,
    doc_generator: &FinalDocGeneratorName,
    needed_file_name: &str
  ) -> Result<(), String> {
    let example_command: &str = match doc_generator {
      FinalDocGeneratorName::Doxygen => "gen-default doxyfile",
      FinalDocGeneratorName::Sphinx => "gen-default sphinx-config"
    };

    return Err(format!(
      "Project [{}] set documentation generator to {}, but is missing its '{}'. Please create '{}'.\n --> The command `{} {}` can be used to do this automatically.",
      self.get_name_for_error_messages(),
      doc_generator.to_str().yellow(),
      format!("{}/{}", self.docs_dir_relative_to_project_root, needed_file_name).yellow(),
      format!("{}/{}", self.docs_dir_relative_to_cwd, needed_file_name).yellow(),
      "gcmake-rust".bright_magenta(),
      example_command.bright_magenta()
    ));
  }

  fn validate_header_names(&self) -> Result<(), String> {
    if let ProjectOutputType::HeaderOnlyLibProject = &self.project_output_type {
      if !self.private_headers.is_empty() {
        return Err(format!(
          "Project [{}] creates a header-only library, but contains private header files. A header-only library can't have private headers. To fix, remove all headers in the project's {}/ directory ({}).",
          self.get_name_for_error_messages().yellow(),
          SRC_DIR_NAME,
          self.src_dir_relative_to_cwd.yellow()
        ));
      }
    }

    for private_header in &self.private_headers {
      let path_without_header_extension: PathBuf = private_header.get_file_path().with_extension("");
      
      if let Some(extension) = path_without_header_extension.extension() {
        if extension != "private" {
          let current_header_path_str: &str = private_header.get_file_path().to_str().unwrap();
          let correct_private_header_path: PathBuf = path_without_header_extension
            .with_extension(format!(
              "private.{}",
              private_header.get_file_path().extension().unwrap().to_str().unwrap()
            ));

          return Err(format!(
            "Project [{}] has a private header file \"{}\" which is missing the '{}' part of its extension. Private files are required to have '{}' before the actual header extension (.h, .hpp, etc.). Try changing the file path like this:\n   From: {}\n   To: {}",
            self.get_name_for_error_messages().yellow(),
            current_header_path_str.yellow(),
            ".private".purple(),
            ".private".purple(),
            current_header_path_str.red(),
            correct_private_header_path.to_str().unwrap().green()
          ));
        }
      }
    }

    // TODO: Ensure public headers don't contain a '.private' extension.

    Ok(())
  }

  fn ensure_doc_generator_correctness(&self, project_load_context: &FinalProjectLoadContext) -> Result<(), String> {
    let doxyfile_in_search_result: Option<PathBuf> = find_doxyfile_in(&self.docs_dir_relative_to_cwd);
    let sphinx_files_search_result: SphinxConfigFiles = find_sphinx_files(&self.docs_dir_relative_to_cwd);

    match &self.documentation {
      None => {
        if let Some(doxyfile_path) = &doxyfile_in_search_result {
          self.warn_unused_doc_config_file(FinalDocGeneratorName::Doxygen, doxyfile_path);
        }
        
        if let Some(index_rst_path) = &sphinx_files_search_result.index_rst {
          self.warn_unused_doc_config_file(FinalDocGeneratorName::Sphinx, index_rst_path);
        }

        if let Some(conf_py_path) = &sphinx_files_search_result.conf_py_in {
          self.warn_unused_doc_config_file(FinalDocGeneratorName::Sphinx, conf_py_path);
        }
      },
      Some(doc_config) => {
        return self.validate_existing_doc_generator_config(
          doc_config,
          doxyfile_in_search_result,
          sphinx_files_search_result,
          project_load_context.about_to_generate_doxyfile,
          project_load_context.about_to_generate_sphinx_files
        )
      }
    }

    Ok(())
  }

  fn validate_existing_doc_generator_config(
    &self,
    doc_config: &FinalDocumentationInfo,
    doxyfile_in_search_result: Option<PathBuf>,
    sphinx_files_search_result: SphinxConfigFiles,
    is_missing_doxyfile_okay: bool,
    are_missing_sphix_files_okay: bool
  ) -> Result<(), String> {
    // For now, the only two supported documentation generators are Doxygen and Sphinx.
    // Since both require a Doxyfile.in, it's fine to move this check out.
    match doxyfile_in_search_result {
      Some(doxyfile_in_pathbuf) => {
        validate_doxyfile_in(&doxyfile_in_pathbuf)?;
      },
      None => {
        if !is_missing_doxyfile_okay {
          return self.err_for_missing_doc_config_file(&doc_config.generator, "Doxyfile.in");
        }
      }
    }

    return match doc_config.generator {
      FinalDocGeneratorName::Doxygen => return Ok(()),
      FinalDocGeneratorName::Sphinx => match sphinx_files_search_result {
        SphinxConfigFiles { conf_py_in: None, .. } => {
          if !are_missing_sphix_files_okay {
            self.err_for_missing_doc_config_file(&doc_config.generator, "conf.py.in")
          }
          else {
            Ok(())
          }
        },
        SphinxConfigFiles { index_rst: None, .. } => {
          if !are_missing_sphix_files_okay {
            self.err_for_missing_doc_config_file(&doc_config.generator, "index.rst")
          }
          else {
            Ok(())
          }
        },
        SphinxConfigFiles { conf_py_in, .. } => {
          validate_conf_py_in(&conf_py_in.unwrap())
        }
      }
    }
  }

  pub fn any_files_contain_cpp2_grammar(&self) -> bool {
    return !self.all_cpp_sources_by_grammar(CppFileGrammar::Cpp2, true).is_empty();
  }

  pub fn pre_build_entry_file(&self) -> Option<&CodeFileInfo> {
    if let Some(pre_build) = self.get_prebuild_script() {
      if let PreBuildScriptType::Exe(pre_build_exe) = pre_build.get_type() {
        return Some(pre_build_exe.get_entry_file())
      }
    }
    return None;
  }

  // Also includes entry files for output items and executable pre-build script.
  pub fn all_cpp_sources_by_grammar(
    &self,
    grammar: CppFileGrammar,
    // Since the pre-build script is able to generate code files, we sometimes need the pre-build
    // entry file to be transformed in a separate step from the rest of the project code.
    should_include_pre_build_entry: bool
  ) -> HashSet<&CodeFileInfo> {
    let mut source_file_set: HashSet<&CodeFileInfo> = self.src_files.iter()
      .filter_map(|code_file_info|
        if code_file_info.uses_cpp2_grammar() {
          Some(code_file_info)
        }
        else {
          None
        }
      )
      .collect();

    for (_, output) in &self.output {
      if let RetrievedCodeFileType::Source(CodeFileLang::Cpp { used_grammar }) = output.entry_file.code_file_type() {
        if grammar == used_grammar {
          source_file_set.insert(output.get_entry_file());
        }
      }
    }

    if should_include_pre_build_entry {
      if let Some(pre_build_entry) = self.pre_build_entry_file() {
        match (grammar, pre_build_entry.uses_cpp2_grammar()) {
          (CppFileGrammar::Cpp1, false)
          | (CppFileGrammar::Cpp2, true) =>
          {
            source_file_set.insert(pre_build_entry);
          },
          _ => ()
        }
      }
    }

    return source_file_set;
  }

  fn ensure_no_file_collision(&self) -> Result<(), String> {
    let existing_normal_cpp_files: HashSet<&CodeFileInfo> = self.all_cpp_sources_by_grammar(CppFileGrammar::Cpp1, true);

    for cpp2_file_info in self.all_cpp_sources_by_grammar(CppFileGrammar::Cpp2, true) {
      let cpp2_file: &Path = cpp2_file_info.get_file_path();
      let generated_file_name: PathBuf = cpp2_file.with_extension("").with_extension(".cpp");

      if existing_normal_cpp_files.contains(&CodeFileInfo::from_path(generated_file_name.as_path(), false)) {
        return Err(format!(
          "Source file conflict! cpp2 file \"{}\" will be used to generate cpp file \"{}\" at build time, but the file \"{}\" already exists. Please rename one of the files to something else.",
          cpp2_file.to_str().unwrap().green(),
          generated_file_name.to_str().unwrap().yellow(),
          generated_file_name.to_str().unwrap().yellow(),
        ));
      }
    }

    Ok(())
  }

  fn validate_features(&self) -> Result<(), String> {
    for (feature_name, feature_config) in self.features.iter() {
      if feature_name.contains(" ") {
        return Err(format!(
          "Invalid feature name \"{}\" given. Feature names cannot contain whitespace.",
          feature_name.yellow()
        ));
      }

      for FinalFeatureEnabler { dep_name, feature_name: feature_name_to_enable } in &feature_config.enables {
        // Dependency feature enablers are checked in the dependency graph's
        // do_additional_project_checks(...) function.
        if dep_name.is_none() && !self.features.contains_key(feature_name) {
          return Err(format!(
            "Feature \"{}\" specifies that it should enable another feature named \"{}\", but the project doesn't define a feature called {}.",
            feature_name.purple(),
            feature_name_to_enable.yellow(),
            feature_name_to_enable.yellow()
          ));
        }
      }
    }

    Ok(())
  }

  fn ensure_valid_icon_config(
    &self,
    item_name: &str,
    target: &CompiledOutputItem
  ) -> Result<(), String> {
    if !target.is_executable_type() && target.windows_icon_relative_to_root_project.is_some() {
      return Err(format!(
        "{} is not an executable, but specifies a windows_icon '{}'. Windows icons can only be specified for executables.",
        item_name,
        target.windows_icon_relative_to_root_project.as_ref().unwrap().to_str().unwrap()
      ));
    }

    Ok(())
  }

  fn validate_installer_config(&self) -> Result<(), String> {
    for (output_name, _) in &self.installer_config.shortcuts {
      match self.find_output_in_whole_tree(output_name) {
        None => return Err(format!(
          "The installer config in project [{}] tries to create a shortcut for executable output '{}', but the project doesn't have an executable output named '{}'.",
          self.get_name_for_error_messages(),
          output_name,
          output_name
        )),
        Some(matching_output) => {
          if !matching_output.is_executable_type() {
            return Err(format!(
              "The installer config in project [{}] tries to create a shortcut for output item '{}', but '{}' is not an executable. Installer shortcuts can only be created for executables.",
              self.get_name_for_error_messages(),
              output_name,
              output_name
            ));
          }
        }
      }
    }

    Ok(())
  }

  fn find_output_in_whole_tree(&self, target_name: &str) -> Option<&CompiledOutputItem> {
    if let Some(found_target) = self.output.get(target_name) {
      return Some(found_target);
    }

    for (_, subproject) in &self.subprojects {
      if let Some(found_target) = subproject.find_output_in_whole_tree(target_name) {
        return Some(found_target);
      }
    }
    return None;
  }

  fn validate_entry_file_path(
    &self,
    item_name: &str,
    output_item: &CompiledOutputItem
  ) -> Result<(), String> {
    let absolute_entry_file_path: PathBuf = absolute_path(
      Path::new(self.get_project_root_dir())
        .join(output_item.get_entry_file().get_file_path())
    )?;
    let entry_file_directory: &Path = absolute_entry_file_path.parent().unwrap();

    if entry_file_directory != self.get_absolute_project_root() {
      let is_in_subdirectory_of_root: bool = entry_file_directory.starts_with(self.get_absolute_project_root());

      if is_in_subdirectory_of_root {
        return Err(format!(
          "The entry_file \"{}\" for {} is in a subdirectory of its project root \"{}\". Entry files can only be placed in the immediate root directory of the project which contains them.",
          absolute_entry_file_path.to_str().unwrap().magenta(),
          item_name.yellow(),
          self.get_absolute_project_root().to_str().unwrap().magenta()
        ));
      }
      else {
        return Err(format!(
          "The entry_file \"{}\" for {} is not in the project's root directory \"{}\". Entry files can only be placed in the immediate root directory of the project which contains them.",
          absolute_entry_file_path.to_str().unwrap().magenta(),
          item_name.yellow(),
          self.get_absolute_project_root().to_str().unwrap().magenta()
        ));
      }
    }
    else if !absolute_entry_file_path.exists() {
      let without_filename: &str = absolute_entry_file_path.parent().unwrap().to_str().unwrap();
      let file_name: &str = absolute_entry_file_path.file_name().unwrap().to_str().unwrap();

      return Err(format!(
        "The entry_file \"{}\" given for {} doesn't exist. Is the file missing or named something else?\n   Expected file at: \"{}/{}\"",
        file_name.magenta(),
        item_name.yellow(),
        without_filename,
        file_name.magenta()
      ));
    }

    Ok(())
  }

  fn validate_target_info(
    &self,
    item_name: &str,
    output_item: &CompiledOutputItem,
    _is_prebuild_script: bool
  ) -> Result<(), String> {
    self.validate_entry_file_type(item_name, output_item)?;
    self.validate_entry_file_path(item_name, output_item)?;

    self.validate_output_specific_build_config(
      item_name,
      output_item.get_build_config_map()
    )?;

    self.ensure_valid_icon_config(item_name, output_item)?;

    Ok(())
  }

  fn validate_entry_file_type(
    &self,
    item_name: &str,
    output_item: &CompiledOutputItem
  ) -> Result<(), String> {
    let entry_file_type: RetrievedCodeFileType = output_item.get_entry_file().code_file_type();

    match *output_item.get_output_type() {
      OutputItemType::Executable => {
        if !entry_file_type.is_source() {
          return Err(format!(
            "The entry_file for {} executable in project '{}' should be a source file, but isn't.",
            item_name,
            self.get_project_base_name()
          ));
        }
      },
      OutputItemType::CompiledLib
        | OutputItemType::StaticLib
        | OutputItemType::SharedLib
        | OutputItemType::HeaderOnlyLib =>
      {
        if !entry_file_type.is_normal_header() {
          return Err(format!(
            "The entry_file for {} library in project '{}' should be a header file, but isn't.",
            item_name,
            self.get_project_base_name()
          ));
        }
      }
    }
    
    Ok(())
  }

  fn validate_project_type_specific_info(&self) -> Result<(), String> {
    match &self.project_output_type {
      ProjectOutputType::ExeProject => (),
      ProjectOutputType::CompiledLibProject => {
        assert!(
          self.output.len() == 1,
          "CompiledLibProject should contain only one output."
        );

        if self.src_files.is_empty() && !self.was_just_created {
          return Err(format!(
            "Project '{}' builds a compiled library '{}', however the project contains no source (.c or .cpp) files. Compiled libraries must contain at least one source file. If this is supposed to be a header-only library, change the output_type to '{}'",
            self.get_project_base_name(),
            self.get_outputs().keys().collect::<Vec<&String>>()[0],
            OutputItemType::HeaderOnlyLib.name_string()
          ));
        }
      },
      ProjectOutputType::HeaderOnlyLibProject => {
        assert!(
          self.output.len() == 1,
          "HeaderOnlyLibProject should contain only one output."
        );

        if !self.src_files.is_empty() {
          return Err(format!(
            "Project '{}' builds a header-only library '{}', however the project contains some source (.c or .cpp) files. Header-only libraries should not have any source files. If this is supposed to be a compiled library, change the output_type to '{}' or another compiled library type.",
            self.get_project_base_name(),
            self.get_outputs().keys().collect::<Vec<&String>>()[0],
            OutputItemType::CompiledLib.name_string()
          ))
        }
      }
    }

    Ok(())
  }

  fn validate_output_specific_build_config(
    &self,
    item_name: &str,
    maybe_build_config_map: &Option<FinalTargetBuildConfigMap>
  ) -> Result<(), String> {
    if maybe_build_config_map.is_none() {
      return Ok(());
    }

    for (build_type_or_all, config_by_compiler) in maybe_build_config_map.as_ref().unwrap() {
      let build_type_name: &str = build_type_or_all.name_string();

      match build_type_or_all {
        TargetSpecificBuildType::AllConfigs => (),
        targeted_build_type => {
          let build_type: BuildType = targeted_build_type.to_general_build_type().unwrap();

          if !self.build_config_map.contains_key(&build_type) {
            return Err(format!(
              "The {} in project '{}' contains a '{}' configuration, but no '{}' build configuration is provided by the toplevel project.",
              item_name,
              self.get_project_base_name(),
              build_type_name,
              build_type_name
            ))
          }
        }
      }

      for (compiler_specifier, _) in config_by_compiler {
        match compiler_specifier {
          BuildConfigCompilerSpecifier::AllCompilers => continue,
          narrowed_specifier => {
            let specific_specifier: SpecificCompilerSpecifier = narrowed_specifier.to_specific().unwrap();

            if !self.supported_compilers.contains(&specific_specifier) {
              let specific_spec_name: &str = specific_specifier.name_string();

              return Err(format!(
                "The '{}' build_config for {} in project '{}' contains a configuration for '{}', but '{}' is not supported by the project. If it should be supported, add '{}' to the supported_compilers list in the toplevel project.",
                build_type_name,
                item_name,
                self.get_project_base_name(),
                specific_spec_name,
                specific_spec_name,
                specific_spec_name
              ))
            }
          }
        }
      }
    }

    Ok(())
  }

  pub fn nested_include_prefix(&self, next_include_prefix: &str) -> String {
    return format!("{}/{}", &self.full_include_prefix, next_include_prefix);
  }

  pub fn has_tests(&self) -> bool {
    !self.tests.is_empty()
  }

  pub fn has_predefined_dependencies(&self) -> bool {
    !self.predefined_dependencies.is_empty()
  }

  pub fn has_any_fetchcontent_dependencies(&self) -> bool {
    let num_needing_fetch: usize = self.predefined_dependencies
      .iter()
      .filter(|(_, dep_info)| dep_info.is_fetchcontent())
      .collect::<HashMap<_, _>>()
      .len();

    return num_needing_fetch > 0;
  }

  pub fn has_gcmake_dependencies(&self) -> bool {
    self.gcmake_dependency_projects.len() > 0
  }

  pub fn needs_fetchcontent(&self) -> bool {
    self.has_gcmake_dependencies() || self.has_any_fetchcontent_dependencies()
  }

  pub fn full_test_name(
    &self,
    test_target_name: &str
  ) -> String {
    return format!("{}{}{}",
      self.get_full_namespaced_project_name(),
      TEST_TARGET_JOIN_STR,
      test_target_name
    );
  }

  pub fn prefix_with_project_namespace(&self, name: &str) -> String {
    return format!("{}::{}", &self.target_namespace_prefix, name);
  }

  pub fn receiver_lib_name(
    &self,
    target_name: &str
  ) -> String {
    return format!("{}_INTERNAL_RECEIVER_LIB", target_name);
  }

  pub fn prebuild_script_name(&self) -> String {
    return format!(
      "PRE_BUILD_SCRIPT_{}",
      self.project_base_name
    )
  }

  pub fn get_features(&self) -> &BTreeMap<String, FinalFeatureConfig> {
    &self.features
  }

  pub fn get_test_framework(&self) -> &Option<FinalTestFramework> {
    &self.test_framework
  }

  pub fn get_outputs(&self) -> &HashMap<String, CompiledOutputItem> {
    &self.output
  }

  pub fn get_documentation_config(&self) -> Option<&FinalDocumentationInfo> {
    self.documentation.as_ref()
  }

  pub fn get_prebuild_script(&self) -> &Option<PreBuildScript> {
    &self.prebuild_script
  }

  pub fn get_project_root_dir(&self) -> &str {
    &self.project_root_dir
  }

  pub fn get_absolute_project_root(&self) -> &Path {
    &self.absolute_project_root.as_path()
  }

  pub fn get_base_include_prefix(&self) -> &str {
    &self.base_include_prefix
  }

  pub fn get_full_include_prefix(&self) -> &str {
    &self.full_include_prefix
  }

  pub fn get_project_base_name(&self) -> &str {
    &self.project_base_name
  }

  pub fn get_full_namespaced_project_name(&self) -> &str {
    &self.full_namespaced_project_name
  }

  pub fn get_name_for_error_messages(&self) -> &str {
    &self.project_name_for_error_messages
  }

  pub fn get_description(&self) -> &str {
    &self.description
  }

  pub fn get_installer_title(&self) -> &str {
    &self.installer_config.title
  }

  pub fn get_installer_shortcuts_config(&self) -> &HashMap<String, FinalShortcutConfig> {
    &self.installer_config.shortcuts
  }

  pub fn get_installer_description(&self) -> &str {
    &self.installer_config.description
  }

  pub fn get_installer_name_prefix(&self) -> &str {
    &self.installer_config.name_prefix
  }

  pub fn get_vendor(&self) -> &str {
    &self.vendor
  }

  pub fn get_src_dir_relative_to_cwd(&self) -> &str {
    &self.src_dir_relative_to_cwd
  }

  pub fn get_src_dir_relative_to_project_root(&self) -> &str {
    &self.src_dir_relative_to_project_root
  }

  pub fn get_include_dir_relative_to_cwd(&self) -> &str {
    &self.include_dir_relative_to_cwd
  }

  pub fn get_include_dir_relative_to_project_root(&self) -> &str {
    &self.include_dir_relative_to_project_root
  }

  pub fn get_docs_dir_relative_to_cwd(&self) -> &str {
    &self.docs_dir_relative_to_cwd
  }

  pub fn get_build_configs(&self) -> &FinalBuildConfigMap {
    &self.build_config_map
  }

  pub fn get_default_build_config(&self) -> &BuildType {
    &self.default_build_config
  }

  pub fn get_language_info(&self) -> &LanguageConfigMap {
    &self.language_config_map
  }

  pub fn has_global_defines(&self) -> bool {
    !self.global_defines.is_empty()
  }

  pub fn is_ipo_enabled_for(&self, build_type: BuildType) -> bool {
    match &self.global_properties {
      None => false,
      Some(global_properties) => global_properties.ipo_enabled_by_default_for.contains(&build_type)
    }
  }

  pub fn are_language_extensions_enabled(&self) -> bool {
    match &self.global_properties {
      None => false,
      Some(global_properties) => global_properties.are_language_extensions_enabled
    }
  }

  pub fn get_global_defines(&self) -> &Vec<CompilerDefine> {
    &self.global_defines
  }

  pub fn get_default_compiled_lib_type(&self) -> DefaultCompiledLibType {
    match &self.global_properties {
      Some(global_props) => global_props.default_compiled_lib_type.clone(),
      None => DefaultCompiledLibType::Shared
    }
  }
  
  pub fn get_test_projects(&self) -> &SubprojectMap {
    &self.tests
  }

  pub fn get_subprojects(&self) -> &SubprojectMap {
    &self.subprojects
  }

  pub fn get_project_type(&self) -> &FinalProjectType {
    &self.project_type
  }

  pub fn get_project_output_type(&self) -> &ProjectOutputType {
    &self.project_output_type
  }

  pub fn get_predefined_dependencies(&self) -> &HashMap<String, Rc<FinalPredefinedDependencyConfig>> {
    &self.predefined_dependencies
  }

  pub fn get_gcmake_dependencies(&self) -> &HashMap<String, Rc<FinalGCMakeDependency>> {
    &self.gcmake_dependency_projects
  }

  pub fn supports_emscripten(&self) -> bool {
    for (_, subproject) in &self.subprojects {
      if !subproject.supports_emscripten() {
        return false;
      }
    }

    for (_, predef_dep) in &self.predefined_dependencies {
      if !predef_dep.supports_emscripten() {
        return false;
      }
    }

    for (_, gcmake_dep) in &self.gcmake_dependency_projects {
      if !gcmake_dep.supports_emscripten() {
        return false;
      }
    }

    return true;
  }

  pub fn can_trivially_cross_compile(&self) -> bool {
    for (_, subproject) in &self.subprojects {
      if !subproject.can_trivially_cross_compile() {
        return false;
      }
    }

    for (_, predef_dep) in &self.predefined_dependencies {
      if !predef_dep.can_trivially_cross_compile() {
        return false;
      }
    }

    for (_, gcmake_dep) in &self.gcmake_dependency_projects {
      if !gcmake_dep.can_trivially_cross_compile() {
        return false;
      }
    }

    return true;
  }
}
