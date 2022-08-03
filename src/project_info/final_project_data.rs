use std::{collections::{HashMap, HashSet}, path::{Path, PathBuf}, io, rc::Rc, fs::{self}};

use crate::project_info::path_manipulation::cleaned_pathbuf;

use super::{path_manipulation::{cleaned_path_str, relative_to_project_root, absolute_path}, final_dependencies::{FinalGCMakeDependency, FinalPredefinedDependencyConfig}, raw_data_in::{RawProject, dependencies::internal_dep_config::AllRawPredefinedDependencies, BuildConfigMap, BuildType, LanguageConfigMap, OutputItemType, PreBuildConfigIn, SpecificCompilerSpecifier, BuildConfigCompilerSpecifier, TargetBuildConfigMap, TargetSpecificBuildType, LinkSection, RawTestFramework, RawInstallerConfig}, final_project_configurables::{FinalProjectType}, CompiledOutputItem, helpers::{parse_subproject_data, parse_root_project_data, populate_files, find_prebuild_script, PrebuildScriptFile, validate_raw_project_outputs, ProjectOutputType, RetrievedCodeFileType, retrieve_file_type, parse_test_project_data}, PreBuildScript, OutputItemLinks, FinalTestFramework, base_include_prefix_for_test, gcmake_constants::{SRC_DIR, INCLUDE_DIR, TEMPLATE_IMPL_DIR, TESTS_DIR, SUBPROJECTS_DIR}, FinalInstallerConfig};

const SUBPROJECT_JOIN_STR: &'static str = "_S_";
const TEST_PROJECT_JOIN_STR: &'static str = "_TP_";
const TEST_TARGET_JOIN_STR: &'static str = "_T_";

pub struct ThreePartVersion (u32, u32, u32);

impl ThreePartVersion {
  pub fn to_string(&self) -> String {
    let Self (major, minor, patch) = self;

    format!("{}.{}.{}", major, minor, patch)
  }

  /*
    Allowed input formats:
      - v0.0.1
      - 0.0.1
  */
  pub fn from_str(full_version_string: &str) -> Option<Self> {
    let usable_version_string = if full_version_string.starts_with('v')
      { &full_version_string[1..] }
      else { full_version_string };

    let mut version_nums: Vec<Result<u32, _>> = usable_version_string
      .split('.')
      .map(|section| section.parse::<u32>())
      .collect();

    if version_nums.len() != 3 {
      return None;
    }

    for maybe_num in &version_nums {
      if maybe_num.is_err() {
        return None;
      }
    }

    return Some(Self(
      version_nums.remove(0).unwrap(),
      version_nums.remove(0).unwrap(),
      version_nums.remove(0).unwrap()
    ));
  }
}

fn resolve_prebuild_script(project_root: &str, pre_build_config: &PreBuildConfigIn) -> Result<Option<PreBuildScript>, String> {
  let merged_script_config = if let Some(script_file) = find_prebuild_script(project_root) {
    Some(match script_file {
      PrebuildScriptFile::Exe(entry_file_pathbuf) => {
        PreBuildScript::Exe(CompiledOutputItem {
          output_type: OutputItemType::Executable,
          entry_file: relative_to_project_root(project_root, entry_file_pathbuf),
          links: match &pre_build_config.link {
            Some(raw_links) => CompiledOutputItem::make_link_map(
              &format!("Pre-build script"),
              &OutputItemType::Executable,
              &LinkSection::Uncategorized(raw_links.clone())
            )?,
            None => OutputItemLinks::new_empty()
          },
          build_config: pre_build_config.build_config.clone(),
          requires_custom_main: false
        })
      },
      PrebuildScriptFile::Python(python_file_pathbuf) => PreBuildScript::Python(
        relative_to_project_root(project_root, python_file_pathbuf)
      )
    })
  }
  else { None };

  return Ok(merged_script_config);
}

pub struct UseableFinalProjectDataGroup {
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

  while path_using.exists() {
    path_using.push("cmake_data.yaml");
    path_using = cleaned_pathbuf(path_using);

    if !path_using.is_file() {
      return Ok(Some(levels_up_checked - 1));
    }

    levels_up_checked += 1;
    path_using.pop();
    path_using.pop();

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
  build_config_map: Rc<BuildConfigMap>,
  language_config_map: Rc<LanguageConfigMap>,
  supported_compilers: Rc<HashSet<SpecificCompilerSpecifier>>
}

pub struct ProjectConstructorConfig {
  pub just_created_library_project_at: Option<String>
}

// NOTE: Link validity is now checked using the DependencyGraph.
pub struct FinalProjectData {
  project_type: FinalProjectType,
  project_output_type: ProjectOutputType,
  project_root: String,
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
  build_config_map: Rc<BuildConfigMap>,
  default_build_config: BuildType,
  language_config_map: Rc<LanguageConfigMap>,
  global_defines: Option<HashSet<String>>,
  base_include_prefix: String,
  full_include_prefix: String,
  src_dir: String,
  include_dir: String,
  template_impls_dir: String,
  pub src_files: Vec<PathBuf>,
  pub include_files: Vec<PathBuf>,
  pub template_impl_files: Vec<PathBuf>,
  subprojects: SubprojectMap,
  test_framework: Option<FinalTestFramework>,
  tests: TestProjectMap,
  output: HashMap<String, CompiledOutputItem>,

  predefined_dependencies: HashMap<String, Rc<FinalPredefinedDependencyConfig>>,
  gcmake_dependency_projects: HashMap<String, Rc<FinalGCMakeDependency>>,

  prebuild_script: Option<PreBuildScript>,
  target_namespace_prefix: String,
  was_just_created: bool
}

impl FinalProjectData {
  pub fn new(
    unclean_given_root: &str,
    dep_config: &AllRawPredefinedDependencies,
    constructor_config: ProjectConstructorConfig
  ) -> Result<UseableFinalProjectDataGroup, ProjectLoadFailureReason> {
    let cleaned_given_root: String = cleaned_path_str(unclean_given_root);

    let levels_below_root: usize = match project_levels_below_root(cleaned_given_root.as_str()) {
      Err(err) => return Err(ProjectLoadFailureReason::Other(
        format!("Error when trying to find project level: {}", err.to_string())
      )),
      Ok(maybe_level) => match maybe_level {
        Some(value) => value,
        None => return Err(ProjectLoadFailureReason::MissingYaml(format!(
          "Failed to determine project level using '{}'",
          &cleaned_given_root
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
      &constructor_config.just_created_library_project_at
        .map(|creation_root| absolute_path(creation_root).unwrap())
    )?);

    root_project.validate_correctness()
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

    // TODO: Resolve the given predefined dependency (which corresponds to the test framework)
    // and use it here.
    let final_test_framework: Option<FinalTestFramework>;

    let language_config: Rc<LanguageConfigMap>;
    let build_config: Rc<BuildConfigMap>;
    let supported_compiler_set: Rc<HashSet<SpecificCompilerSpecifier>>;

    match &parent_project_info {
      None => {
        raw_project = parse_root_project_data(&unclean_project_root)?;
        language_config = Rc::new(raw_project.languages.clone());
        build_config = Rc::new(raw_project.build_configs.clone());
        supported_compiler_set = Rc::new(raw_project.supported_compilers.clone());
        full_namespaced_project_name = raw_project.name.clone();
        project_type = FinalProjectType::Root;
        final_test_framework = match &raw_project.test_framework {
          None => None,
          Some(raw_framework_info) => {
            // REFACTOR: Pretty sure I can refactor this somehow.
            let test_framework_lib: Rc<FinalPredefinedDependencyConfig> = FinalPredefinedDependencyConfig::new(
              all_dep_config,
              raw_framework_info.lib_config(),
              raw_framework_info.name()
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
        ..
      }) => {
        language_config = Rc::clone(language_config_map);
        supported_compiler_set = Rc::clone(supported_compilers);
        build_config = Rc::clone(build_config_map);

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
        ..
      }) => {
        language_config = Rc::clone(language_config_map);
        supported_compiler_set = Rc::clone(supported_compilers);
        build_config = Rc::clone(build_config_map);

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

    let project_root: String = cleaned_path_str(&unclean_project_root).to_string();
    let project_vendor: String = raw_project.vendor.clone();

    let project_src_dir = format!(
      "{}/{}/{}",
      &project_root,
      SRC_DIR,
      &full_include_prefix
    );

    let project_include_dir = format!(
      "{}/{}/{}",
      &project_root,
      INCLUDE_DIR,
      &full_include_prefix
    );

    let project_template_impls_dir = format!(
      "{}/{}/{}",
      &project_root,
      TEMPLATE_IMPL_DIR,
      &full_include_prefix
    );

    let mut test_project_map: SubprojectMap = SubprojectMap::new();

    let project_test_dir_path: PathBuf = PathBuf::from(format!(
      "{}/{}",
      &project_root,
      TESTS_DIR
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
              supported_compilers: Rc::clone(&supported_compiler_set)
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
      &project_root,
      SUBPROJECTS_DIR
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
              language_config_map: Rc::clone(&language_config)
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
        let dep_path: String = format!("{}/dep/{}", &project_root, &dep_name);

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

      output_items.insert(
        output_name.to_owned(),
        CompiledOutputItem::from(output_name, raw_output_item)
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
          dep_name
        )
          .map_err(ProjectLoadFailureReason::Other)?;

        predefined_dependencies.insert(dep_name.clone(), Rc::new(finalized_dep));
      }
    }

    let prebuild_script = resolve_prebuild_script(
      &project_root,
      raw_project.prebuild_config.as_ref().unwrap_or(&PreBuildConfigIn {
        link: None,
        build_config: None
      })
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
      },
      Some(RawInstallerConfig { title, description, name_prefix }) => FinalInstallerConfig {
        title: title.clone().unwrap_or(raw_project.name.clone()),
        description: description.clone().unwrap_or(raw_project.description.clone()),
        name_prefix: name_prefix.clone().unwrap_or(raw_project.name.clone())
      }
    };

    let project_name_for_error_messages: String = full_namespaced_project_name
      .split(SUBPROJECT_JOIN_STR)
      .collect::<Vec<&str>>()
      .join(" => ")
      .split(TEST_PROJECT_JOIN_STR)
      .collect::<Vec<&str>>()
      .join(" -> ");

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
      global_defines: raw_project.global_defines,
      build_config_map: build_config,
      default_build_config: raw_project.default_build_type,
      language_config_map: language_config,
      supported_compilers: supported_compiler_set,
      project_type,
      project_output_type,
      absolute_project_root: absolute_path(&project_root)
        .map_err(ProjectLoadFailureReason::Other)?,
      project_root,
      src_dir: project_src_dir,
      include_dir: project_include_dir,
      template_impls_dir: project_template_impls_dir,
      src_files: Vec::<PathBuf>::new(),
      include_files: Vec::<PathBuf>::new(),
      template_impl_files: Vec::<PathBuf>::new(),
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

    populate_files(
      Path::new(&finalized_project_data.src_dir),
      &mut finalized_project_data.src_files
    )
      .map_err(|err| ProjectLoadFailureReason::Other(err.to_string()))?;

    populate_files(
      Path::new(&finalized_project_data.include_dir),
      &mut finalized_project_data.include_files
    )
      .map_err(|err| ProjectLoadFailureReason::Other(err.to_string()))?;

    populate_files(
      Path::new(&finalized_project_data.template_impls_dir),
      &mut finalized_project_data.template_impl_files
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

  pub fn mark_just_created(&mut self, was_just_created: bool) {
    self.was_just_created = was_just_created;
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

  fn ensure_language_config_correctness(&self) -> Result<(), String> {
    let LanguageConfigMap { c, cpp } = self.get_language_info();

    match c.standard {
      99 | 11 | 17 => (),
      standard => return Err(format!("C Language standard must be one of [99, 11, 17], but {} was given", standard))
    }

    match cpp.standard {
      11 | 14 | 17 | 20 => (),
      standard => return Err(format!("C++ Language standard must be one of [11, 14, 17, 20], but {} was given", standard))
    }

    Ok(())
  }

  fn ensure_build_config_correctness(&self) -> Result<(), String> {
    for (build_type, by_compiler_map) in self.get_build_configs() {
      for (config_compiler, _) in by_compiler_map {
        if let Some(specific_compiler) = config_compiler.to_specific() {
          if !self.supported_compilers.contains(&specific_compiler) {
            let compiler_name: &str = specific_compiler.name_string();

            return Err(format!(
              "Config Issue: '{}' build config defines a section for {}, but {} is not in the supported_compilers list. To fix, either remove the {} section or add {} to the supported_compilers list for this project.",
              build_type.name_string(),
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

  fn validate_correctness(&self) -> Result<(), String> {
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

    for (_, test_project) in &self.tests {
      if let ProjectOutputType::ExeProject = &test_project.project_output_type {
        test_project.validate_correctness()?;
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
      subproject.validate_correctness()?;
    }

    self.ensure_language_config_correctness()?;
    self.ensure_build_config_correctness()?;
    self.validate_project_type_specific_info()?;

    for (output_name, output_item) in &self.output {
      self.validate_entry_file_type(
        output_name,
        output_item,
        false
      )?;

      self.validate_output_specific_build_config(
        output_name,
        output_item.get_build_config_map(),
        false
      )?;
    }

    if let Some(existing_script) = &self.prebuild_script {
      match existing_script {
        PreBuildScript::Exe(script_exe_config) => {
          let the_item_name: String = format!("{}'s pre-build script", self.get_project_base_name());

          self.validate_entry_file_type(
            &the_item_name,
            script_exe_config,
            true
          )?;

          self.validate_output_specific_build_config(
            &the_item_name,
            script_exe_config.get_build_config_map(),
            true
          )?;

        },
        PreBuildScript::Python(_) => ()
      }
    }

    Ok(())
  }

  fn validate_entry_file_type(
    &self,
    output_name: &str,
    output_item: &CompiledOutputItem,
    is_prebuild_script: bool
  ) -> Result<(), String> {
    let entry_file_type: RetrievedCodeFileType = retrieve_file_type(output_item.get_entry_file());
    let item_string: String = if is_prebuild_script
      { String::from("prebuild script") }
      else { format!("output item '{}'", output_name )};

    match *output_item.get_output_type() {
      OutputItemType::Executable => {
        if entry_file_type != RetrievedCodeFileType::Source {
          return Err(format!(
            "The entry_file for executable {} in project '{}' should be a source file, but isn't.",
            item_string,
            self.get_project_base_name()
          ));
        }
      },
      OutputItemType::CompiledLib
        | OutputItemType::StaticLib
        | OutputItemType::SharedLib
        | OutputItemType::HeaderOnlyLib =>
      {
        if entry_file_type != RetrievedCodeFileType::Header {
          return Err(format!(
            "The entry_file for library {} in project '{}' should be a header file, but isn't.",
            item_string,
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
    output_name: &str,
    maybe_build_config_map: &Option<TargetBuildConfigMap>,
    is_prebuild_script: bool
  ) -> Result<(), String> {
    if maybe_build_config_map.is_none() {
      return Ok(());
    }

    for (build_type_or_all, config_by_compiler) in maybe_build_config_map.as_ref().unwrap() {
      let build_type_name: &str = build_type_or_all.name_string();
      let item_string: String = if is_prebuild_script
        { String::from("prebuild script") }
        else { format!("output item '{}'", output_name )};

      match build_type_or_all {
        TargetSpecificBuildType::AllConfigs => (),
        targeted_build_type => {
          let build_type: BuildType = targeted_build_type.to_general_build_type().unwrap();

          if !self.build_config_map.contains_key(&build_type) {
            return Err(format!(
              "The {} in project '{}' contains a '{}' configuration, but no '{}' build configuration is provided by the toplevel project.",
              &item_string,
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
                &item_string,
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

  pub fn has_predefined_fetchcontent_ready_dependencies(&self) -> bool {
    let num_needing_fetch: usize = self.predefined_dependencies
      .iter()
      .filter(|(_, dep_info)| dep_info.is_auto_fetchcontent_ready())
      .collect::<HashMap<_, _>>()
      .len();

    return num_needing_fetch > 0;
  }

  pub fn has_gcmake_dependencies(&self) -> bool {
    self.gcmake_dependency_projects.len() > 0
  }

  pub fn has_any_fetchcontent_ready_dependencies(&self) -> bool {
    self.has_gcmake_dependencies() || self.has_predefined_fetchcontent_ready_dependencies()
  }

  pub fn fetchcontent_dep_names(&self) -> impl Iterator<Item = &String> {
    return self.predefined_dependencies
      .iter()
      .filter_map(|(dep_name, dep_info)| {
        if dep_info.is_auto_fetchcontent_ready()
          { Some(dep_name) }
          else { None }
      })
      .chain(self.gcmake_dependency_projects.keys());
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

  pub fn get_test_framework(&self) -> &Option<FinalTestFramework> {
    &self.test_framework
  }

  pub fn get_outputs(&self) -> &HashMap<String, CompiledOutputItem> {
    &self.output
  }

  pub fn get_prebuild_script(&self) -> &Option<PreBuildScript> {
    &self.prebuild_script
  }

  pub fn get_project_root(&self) -> &str {
    &self.project_root
  }

  pub fn get_absolute_project_root(&self) -> &str {
    &self.absolute_project_root.to_str().unwrap()
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

  pub fn get_installer_description(&self) -> &str {
    &self.installer_config.description
  }

  pub fn get_installer_name_prefix(&self) -> &str {
    &self.installer_config.name_prefix
  }

  pub fn get_vendor(&self) -> &str {
    &self.vendor
  }

  pub fn get_src_dir(&self) -> &str {
    &self.src_dir
  }

  pub fn get_include_dir(&self) -> &str {
    &self.include_dir
  }

  pub fn get_template_impl_dir(&self) -> &str {
    &self.template_impls_dir
  }

  pub fn get_build_configs(&self) -> &BuildConfigMap {
    &self.build_config_map
  }

  pub fn get_default_build_config(&self) -> &BuildType {
    &self.default_build_config
  }

  pub fn get_language_info(&self) -> &LanguageConfigMap {
    &self.language_config_map
  }

  pub fn get_global_defines(&self) -> &Option<HashSet<String>> {
    &self.global_defines
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
}

