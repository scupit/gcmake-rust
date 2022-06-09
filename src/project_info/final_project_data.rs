use std::{collections::{HashMap, HashSet}, path::{Path, PathBuf}, io, rc::Rc};

use crate::project_info::path_manipulation::cleaned_pathbuf;

use super::{path_manipulation::{cleaned_path_str, relative_to_project_root, find_first_dir_named, absolute_path}, final_dependencies::{FinalGCMakeDependency, FinalPredefinedDependencyConfig, FinalPredepInfo}, raw_data_in::{RawProject, ProjectLike, dependencies::internal_dep_config::AllRawPredefinedDependencies, BuildConfigMap, BuildType, LanguageConfigMap, CompiledItemType, PreBuildConfigIn, SpecificCompilerSpecifier, ProjectMetadata, BuildConfigCompilerSpecifier, TargetBuildConfigMap, TargetSpecificBuildType}, final_project_configurables::{FinalProjectType, SubprojectOnlyOptions}, CompiledOutputItem, helpers::{create_subproject_data, create_project_data, validate_raw_project, populate_files, find_prebuild_script, PrebuildScriptFile, parse_project_metadata}, PreBuildScript};

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
          output_type: CompiledItemType::Executable,
          entry_file: relative_to_project_root(project_root, entry_file_pathbuf),
          links: match &pre_build_config.link {
            Some(raw_links) => Some(CompiledOutputItem::make_link_map(raw_links)?),
            None => None
          },
          build_config: pre_build_config.build_config.clone()
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

fn project_level(
  clean_path_root: &str,
  include_prefix: &str
) -> io::Result<Option<usize>> {
  let search_dir: String = format!("{}/include", clean_path_root);

  if let Some(dirty_include_path) = find_first_dir_named(Path::new(&search_dir), include_prefix)? {
    let include_path: PathBuf = cleaned_pathbuf(dirty_include_path);
    let path_components: Vec<_> = include_path.components().collect();

    let include_position: usize = include_path.components()
      .position(|section| section.as_os_str().to_str().unwrap() == "include")
      .unwrap();

    let level: usize = path_components.len() - include_position - 2;

    // println!("include_path path: {}", include_path.to_str().unwrap());
    // println!(
    //   "include pos: {}, num comps: {}, level: {}",
    //   include_position,
    //   path_components.len(),
    //   level
    // );
    
    return Ok(Some(level));
  }
  
  Ok(None)
}

type SubprojectMap = HashMap<String, Rc<FinalProjectData>>;

pub enum ProjectLoadFailureReason {
  MissingYaml(String),
  Other(String)
}

impl ProjectLoadFailureReason {
  pub fn extract_message(self) -> String {
    match self {
      Self::MissingYaml(msg) => msg,
      Self::Other(msg) => msg
    }
  }
}

pub enum DependencySearchMode {
  AsParent,
  AsSubproject
}

struct ParentProjectInfo {
  include_prefix: String,
  target_namespace_prefix: String
}

pub struct FinalProjectData {
  project_type: FinalProjectType,
  project_root: String,
  absolute_project_root: PathBuf,
  pub version: ThreePartVersion,
  // project: RawProject,
  supported_compilers: HashSet<SpecificCompilerSpecifier>,
  project_name: String,
  build_config_map: BuildConfigMap,
  default_build_config: BuildType,
  language_config_map: LanguageConfigMap,
  global_defines: Option<HashSet<String>>,
  base_include_prefix: String,
  full_include_prefix: String,
  src_dir: String,
  include_dir: String,
  template_impls_dir: String,
  pub src_files: Vec<PathBuf>,
  pub include_files: Vec<PathBuf>,
  pub template_impl_files: Vec<PathBuf>,
  // subproject_names: HashSet<String>,
  // subprojects: Vec<FinalProjectData>,
  subprojects: SubprojectMap,
  output: HashMap<String, CompiledOutputItem>,
  predefined_dependencies: HashMap<String, FinalPredefinedDependencyConfig>,
  gcmake_dependency_projects: HashMap<String, FinalGCMakeDependency>,
  prebuild_script: Option<PreBuildScript>,
  target_namespace_prefix: String
}

impl FinalProjectData {

  pub fn new(
    unclean_given_root: &str,
    dep_config: &AllRawPredefinedDependencies
  ) -> Result<UseableFinalProjectDataGroup, ProjectLoadFailureReason> {
    let metadata: ProjectMetadata = parse_project_metadata(unclean_given_root)?;
    let cleaned_given_root: String = cleaned_path_str(unclean_given_root);

    let level: usize = match project_level(cleaned_given_root.as_str(), &metadata.include_prefix) {
      Err(err) => return Err(ProjectLoadFailureReason::Other(
        format!("Error when trying to find project level: {}", err.to_string())
      )),
      Ok(maybe_level) => match maybe_level {
        Some(value) => value,
        None => return Err(ProjectLoadFailureReason::Other(format!(
          "Unable to find valid include directory with prefix {} in {}",
          &metadata.include_prefix,
          &cleaned_given_root
        )))
      }
    };

    let mut real_project_root_using: PathBuf = PathBuf::from(&cleaned_given_root);

    if level > 0 {
      // Current project is <level> levels deep. Need to go back <level> * 2 dirs, since subprojects
      // are nested in the 'subprojects/<subproject name>' directory
      for _ in 0..(level * 2) {
        real_project_root_using.push("..");
      }

      real_project_root_using = real_project_root_using;
    }


    let root_project: Rc<FinalProjectData> = Rc::new(Self::create_new(
      real_project_root_using.to_str().unwrap(),
      None,
      dep_config
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
    parent_project_info: Option<ParentProjectInfo>,
    all_dep_config: &AllRawPredefinedDependencies
  ) -> Result<FinalProjectData, ProjectLoadFailureReason> {
    // NOTE: Subprojects are still considered whole projects, however they are not allowed to specify
    // top level build configuration data. This means that language data, build configs, etc. are not
    // defined in subprojects, and shouldn't be written. Build configuration related data is inherited
    // from the parent project.
    let raw_project: RawProject;
    let project_type: FinalProjectType;

    if parent_project_info.is_some() {
      raw_project = create_subproject_data(&unclean_project_root)?.into();
      project_type = FinalProjectType::Subproject(SubprojectOnlyOptions { })
    } else {
      raw_project = create_project_data(&unclean_project_root)?;
      project_type = FinalProjectType::Root;
    };

    if let Err(err_message) = validate_raw_project(&raw_project) {
      return Err(ProjectLoadFailureReason::Other(err_message));
    }

    let full_include_prefix: String;
    let target_namespace_prefix: String;

    if let Some(parent_project) = parent_project_info {
      full_include_prefix = format!(
        "{}/{}",
        parent_project.include_prefix,
        raw_project.get_include_prefix()
      );
      target_namespace_prefix = parent_project.target_namespace_prefix;
    }
    else {
      full_include_prefix = raw_project.get_include_prefix().to_string();
      target_namespace_prefix = raw_project.get_name().to_string();
    }

    let project_root: String = cleaned_path_str(&unclean_project_root).to_string();

    let src_dir = format!("{}/src/{}", &project_root, &full_include_prefix);
    let include_dir = format!("{}/include/{}", &project_root, &full_include_prefix);
    let template_impls_dir = format!("{}/template-impl/{}", &project_root, &full_include_prefix);

    let mut subprojects: SubprojectMap = HashMap::new();
    // let mut subprojects: Vec<FinalProjectData> = Vec::new();
    // let mut subproject_names: HashSet<String> = HashSet::new();

    if let Some(dirnames) = raw_project.get_subproject_dirnames() {
      for subproject_dirname in dirnames {
        let full_subproject_dir = format!("{}/subprojects/{}", &project_root, subproject_dirname);
        let mut new_subproject: FinalProjectData = Self::create_new(
          &full_subproject_dir,
          Some(ParentProjectInfo {
            include_prefix: full_include_prefix.clone(),
            target_namespace_prefix: target_namespace_prefix.clone()
          }),
          all_dep_config
        )?;

        // Subprojects must inherit these from their parent project in order to properly
        // set compiler flags and other properties per output item.
        new_subproject.build_config_map = raw_project.build_configs.clone();
        new_subproject.language_config_map = raw_project.languages.clone();
        new_subproject.supported_compilers = raw_project.supported_compilers.clone();

        subprojects.insert(
          subproject_dirname.clone(),
          Rc::new(new_subproject)
        );
      }
    }

    let mut gcmake_dependency_projects: HashMap<String, FinalGCMakeDependency> = HashMap::new();

    if let Some(gcmake_dep_map) = &raw_project.gcmake_dependencies {
      for (dep_name, dep_config) in gcmake_dep_map {
        let dep_path: String = format!("{}/dep/{}", &project_root, &dep_name);

        let maybe_dep_project: Option<Rc<FinalProjectData>> = if Path::new(&dep_path).exists() {
          Some(Rc::new(Self::create_new(
            &dep_path,
            None,
            all_dep_config
          )?))
        }
        else { None };

        gcmake_dependency_projects.insert(
          dep_name.clone(),
          FinalGCMakeDependency::new(
            &dep_name,
            dep_config,
            maybe_dep_project
          ).map_err(ProjectLoadFailureReason::Other)?
        );
      }
    }

    let mut output_items: HashMap<String, CompiledOutputItem> = HashMap::new();

    for (output_name, raw_output_item) in raw_project.get_output() {
      output_items.insert(
        output_name.to_owned(),
        CompiledOutputItem::from(raw_output_item)
          .map_err(ProjectLoadFailureReason::Other)?
      );
    }

    let mut predefined_dependencies: HashMap<String, FinalPredefinedDependencyConfig> = HashMap::new();

    if let Some(pre_deps) = &raw_project.predefined_dependencies {
      for (dep_name, user_given_config) in pre_deps {
        let finalized_dep = FinalPredefinedDependencyConfig::new(
          all_dep_config,
          user_given_config,
          dep_name
        )
          .map_err(ProjectLoadFailureReason::Other)?;

        predefined_dependencies.insert(dep_name.clone(), finalized_dep);
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

    let mut finalized_project_data = FinalProjectData {
      project_name: raw_project.name.to_string(),
      version: maybe_version.unwrap(),
      full_include_prefix,
      base_include_prefix: raw_project.get_include_prefix().to_string(),
      global_defines: raw_project.global_defines,
      build_config_map: raw_project.build_configs,
      default_build_config: raw_project.default_build_type,
      language_config_map: raw_project.languages,
      supported_compilers: raw_project.supported_compilers,
      project_type,
      absolute_project_root: absolute_path(&project_root)
        .map_err(ProjectLoadFailureReason::Other)?,
      project_root,
      src_dir,
      include_dir,
      template_impls_dir,
      src_files: Vec::<PathBuf>::new(),
      include_files: Vec::<PathBuf>::new(),
      template_impl_files: Vec::<PathBuf>::new(),
      subprojects,
      output: output_items,
      predefined_dependencies,
      gcmake_dependency_projects,
      prebuild_script,
      target_namespace_prefix
    };

    finalized_project_data.populate_used_components()
      .map_err(ProjectLoadFailureReason::Other)?;

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
    None
  }

  // Component-based predefined dependencies need to be told which of their components have been used
  // for the entire project, not per target. Why? find_package requires components to be specified
  // when called (not when linking libraries to target), and also likely populates the library
  // in such a way where the component names are not the same as the targets (or variables) CMake
  // needs to use to link to.
  fn populate_used_components(&mut self) -> Result<(), String> {
    for (dep_name, predefined_dep) in &mut self.predefined_dependencies {
      if let FinalPredepInfo::CMakeComponentsModule(components_dep) = predefined_dep.mut_predef_dep_info() {
        for (_, output_item) in &self.output {
          if let Some(links) = output_item.get_links() {
            if let Some(linked_component_names) = links.get(dep_name) {
              components_dep.mark_multiple_components_used(dep_name, linked_component_names.iter())?;
            }
          }
        }

        if let Some(prebuild_script) = &self.prebuild_script {
          if let PreBuildScript::Exe(CompiledOutputItem { links: Some(links), ..}) = prebuild_script {
            if let Some(linked_component_names) = links.get(dep_name) {
              components_dep.mark_multiple_components_used(dep_name, linked_component_names.iter())?;
            }
          }
        }
      }
    }

    Ok(())
  }

  fn ensure_links_are_valid(
    &self,
    item_name: &str,
    links: &Option<HashMap<String, Vec<String>>>,
    is_prebuild_script: bool
  ) -> Result<(), String> {
    if let Some(link_map) = links {
      // Each library linked to an output item should be member of a subproject or dependency
      // project. This loop checks that each of the referenced sub/dependency project names
      // exist and if they do, that the linked libraries from withing those projects exist
      // as well.
      for (project_name_containing_libraries, lib_names_linking) in link_map {
        // Check if it's linked to a subproject
        if let Some(matching_subproject) = self.subprojects.get(project_name_containing_libraries) {
          if is_prebuild_script {
            return Err(format!(
              "{}'s pre-build script tried to link to a library in subproject '{}', but pre-build scripts can't link to subprojects.",
              self.get_project_name(),
              matching_subproject.get_project_name()
            ));
          }
          else {
            for lib_name_linking in lib_names_linking {
              if !matching_subproject.has_library_output_named(lib_name_linking) {
                return Err(format!(
                  "Output item '{}' in project '{}' tries to link to a nonexistent library '{}' in subproject '{}'.",
                  item_name,
                  self.get_project_name(),
                  lib_name_linking,
                  project_name_containing_libraries
                ));
              }
            }
          }
        }
        // Check if it's linked to a predefined dependency
        else if let Some(final_dep) = self.predefined_dependencies.get(project_name_containing_libraries) {
          for lib_name_linking in lib_names_linking {
            match final_dep.predefined_dep_info() {
              FinalPredepInfo::Subdirectory(subdir_dep) => {
                if !subdir_dep.has_target_named(lib_name_linking) {
                  return Err(format!(
                    "Output item '{}' in project '{}' tries to link to a nonexistent target '{}' in predefined dependency '{}'.",
                    item_name,
                    self.get_project_name(),
                    lib_name_linking,
                    project_name_containing_libraries
                  ))
                }
              },
              FinalPredepInfo::CMakeComponentsModule(components_dep) => {
                if !components_dep.has_component_named(lib_name_linking) {
                  return Err(format!(
                    "Output item '{}' in project '{}' tries to link to a nonexistent component '{}' in predefined dependency '{}'.",
                    item_name,
                    self.get_project_name(),
                    lib_name_linking,
                    project_name_containing_libraries
                  ))
                }
              },
              FinalPredepInfo::CMakeModule(find_module_dep) => {
                if !find_module_dep.has_target_named(lib_name_linking) {
                  return Err(format!(
                    "Output item '{}' in project '{}' tries to link to a nonexistent target '{}' in predefined dependency '{}'.",
                    item_name,
                    self.get_project_name(),
                    lib_name_linking,
                    project_name_containing_libraries
                  ))
                }
              }
            }
            
          }
        }
        else if let Some(final_gcmake_dep) = self.gcmake_dependency_projects.get(project_name_containing_libraries) {
          for lib_name_linking in lib_names_linking {
            if final_gcmake_dep.get_linkable_target_name(lib_name_linking)?.is_none() {
              return Err(format!(
                "Output item '{}' in project '{}' tries to link to a nonexistent target '{}' in gcmake dependency '{}'.",
                item_name,
                self.get_project_name(),
                lib_name_linking,
                project_name_containing_libraries
              ))
            }
          }
        }
        else {
          return Err(format!(
            "Output item '{}' in project '{}' tries to link to libraries in a project named '{}', however that project doesn't exist.",
            item_name,
            self.get_project_name(),
            project_name_containing_libraries
          ));
        }
      }
    }

    Ok(())
  }

  fn ensure_language_config_correctness(&self) -> Result<(), String> {
    let LanguageConfigMap {
      C,
      Cpp
    } = self.get_language_info();

    match C.standard {
      99 | 11 | 17 => (),
      standard => return Err(format!("C Language standard must be one of [99, 11, 17], but {} was given", standard))
    }

    match Cpp.standard {
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
    if self.get_project_name().contains(' ') {
      return Err(format!(
        "Project name cannot contain spaces, but does. (Currently: {})",
        self.get_project_name()
      ));
    }

    if self.get_full_include_prefix().contains(' ') {
      return Err(format!(
        "Project 'include prefix' cannot contain spaces, but does. (Currently: {})",
        self.get_full_include_prefix()
      ));
    }

    for (_, subproject) in &self.subprojects {
      subproject.validate_correctness()?;
    }

    self.ensure_language_config_correctness()?;
    self.ensure_build_config_correctness()?;

    for (output_name, output_item) in &self.output {
      self.ensure_links_are_valid(
        output_name,
        output_item.get_links(),
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
          let the_item_name: String = format!("{}'s pre-build script", self.get_project_name());

          self.ensure_links_are_valid(
            &the_item_name,
            script_exe_config.get_links(),
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
              self.get_project_name(),
              build_type_name,
              build_type_name
            ))
          }
        }
      }

      for (compiler_specifier, _) in config_by_compiler {
        match compiler_specifier {
          BuildConfigCompilerSpecifier::All => continue,
          narrowed_specifier => {
            let specific_specifier: SpecificCompilerSpecifier = narrowed_specifier.to_specific().unwrap();

            if !self.supported_compilers.contains(&specific_specifier) {
              let specific_spec_name: &str = specific_specifier.name_string();

              return Err(format!(
                "The '{}' build_config for {} in project '{}' contains a configuration for '{}', but '{}' is not supported by the project. If it should be supported, add '{}' to the supported_compilers list in the toplevel project.",
                build_type_name,
                &item_string,
                self.get_project_name(),
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

  pub fn has_library_output_named(&self, lib_name: &str) -> bool {
    return match self.get_outputs().get(lib_name) {
      Some(output_item) => output_item.is_library_type(),
      None => false
    }
  }

  pub fn has_subprojects(&self) -> bool {
    !self.subprojects.is_empty()
  }

  pub fn has_predefined_dependencies(&self) -> bool {
    !self.predefined_dependencies.is_empty()
  }

  pub fn has_predefined_fetchable_dependencies(&self) -> bool {
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

  pub fn has_any_fetchcontent_dependencies(&self) -> bool {
    self.has_gcmake_dependencies() || self.has_predefined_fetchable_dependencies()
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

  // These are guaranteed to be valid since all links are checked when gcmake configures the project.
  // TODO: Refactor this. It can probably be more efficient.
  pub fn get_namespaced_library_target_names(
    &self,
    search_mode: DependencySearchMode,
    namespace_prefix: &str,
    item_names: &Vec<String>,
  ) -> Result<Option<Vec<String>>, String> {
    if let DependencySearchMode::AsParent = search_mode {
      if let Some(predef_dep) = self.predefined_dependencies.get(namespace_prefix) {
        match predef_dep.predefined_dep_info() {
          FinalPredepInfo::Subdirectory(subdir_dep) => {
            return Ok(Some(
              item_names
                .iter()
                .map(|base_name| {
                  subdir_dep.get_linkable_target_name(base_name)
                    .unwrap()
                    .to_string()
                })
                .collect()
            ))
          },
          FinalPredepInfo::CMakeComponentsModule(components_dep) => {
            // Here, 'item_names' contains the list of components imported. The components themselves
            // are not linked directly to targets, which is why they aren't passed to the components_dep.
            return Ok(Some(vec![components_dep.linkable_string()]));
          },
          FinalPredepInfo::CMakeModule(find_module_dep) => {
            return Ok(Some(
              item_names
                .iter()
                .map(|base_name| {
                  find_module_dep.get_linkable_target_name(base_name)
                    .unwrap()
                    .to_string()
                })
                .collect()
            ))
          }
        }
      }

      if let Some(gcmake_dep) = self.gcmake_dependency_projects.get(namespace_prefix) {
        let mut namespaced_list: Vec<String> = Vec::new();
        
        for base_name in item_names {
            namespaced_list.push(
              gcmake_dep
                .get_linkable_target_name(base_name)?
                .unwrap()
            );
        }

        return Ok(Some(namespaced_list));
      }
    }

    if let Some(matching_subproject) = self.subprojects.get(namespace_prefix) {
      let mut namespaced_list: Vec<String> = Vec::new();
      
      for base_name in item_names {
          namespaced_list.push(
            matching_subproject
              .get_namespaced_public_linkable_target_name(base_name)?
              .unwrap()
          );
      }

      return Ok(Some(namespaced_list));      
    }

    Ok(None)
  }

  pub fn prefix_with_namespace(&self, name: &str) -> String {
    format!("{}::{}", self.target_namespace_prefix, name)
  }

  pub fn get_namespaced_public_linkable_target_name(&self, base_name: &str) -> Result<Option<String>, String> {
    if let Some(output) = self.output.get(base_name) {
      return if output.is_library_type() {
        Ok(Some(self.prefix_with_namespace(base_name)))
      }
      else {
        Err(format!(
          "Tried to link to executable target '{}' ({}) in project '{}' (project namespace: {}), but you can only link to library targets.",
          base_name,
          self.prefix_with_namespace(base_name),
          self.get_project_name(),
          &self.target_namespace_prefix
        ))
      }
    }

    for (_, subproject) in self.get_subprojects() {
      if let Some(namespaced_target_name) = subproject.get_namespaced_public_linkable_target_name(base_name)? {
        return Ok(Some(namespaced_target_name));
      }
    }

    Ok(None)
  }

  pub fn get_subproject_names(&self) -> HashSet<String> {
    self.subprojects.iter()
      .map(|(subproject_name, _)| subproject_name.to_owned())
      .collect()
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

  pub fn get_project_name(&self) -> &str {
    &self.project_name
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
  
  pub fn get_subprojects(&self) -> &SubprojectMap {
    &self.subprojects
  }

  pub fn get_project_type(&self) -> &FinalProjectType {
    &self.project_type
  }

  pub fn get_predefined_dependencies(&self) -> &HashMap<String, FinalPredefinedDependencyConfig> {
    &self.predefined_dependencies
  }

  pub fn get_gcmake_dependencies(&self) -> &HashMap<String, FinalGCMakeDependency> {
    &self.gcmake_dependency_projects
  }
}

