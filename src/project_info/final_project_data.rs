use std::{collections::{HashMap, HashSet}, path::{Path, PathBuf}};

use super::{path_manipulation::{cleaned_path_str, cleaned_pathbuf, relative_to_project_root}, final_dependencies::FinalPredefinedDependency, raw_data_in::{RawProject, RawSubproject, ProjectLike, dependencies::internal_dep_config::AllPredefinedDependencies, BuildConfigMap, BuildType, LanguageMap, CompiledItemType, PreBuildConfigIn}, final_project_configurables::{FinalProjectType, SubprojectOnlyOptions}, CompiledOutputItem, helpers::{create_subproject_data, create_project_data, validate_raw_project, populate_files, find_prebuild_script, PrebuildScriptFile}, PreBuildScript};

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
          }
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

pub struct FinalProjectData {
  project_type: FinalProjectType,
  project_root: String,
  project: RawProject,
  src_dir: String,
  include_dir: String,
  template_impls_dir: String,
  pub src_files: Vec<PathBuf>,
  pub include_files: Vec<PathBuf>,
  pub template_impl_files: Vec<PathBuf>,
  // subproject_names: HashSet<String>,
  // subprojects: Vec<FinalProjectData>,
  subprojects: HashMap<String, FinalProjectData>,
  output: HashMap<String, CompiledOutputItem>,
  predefined_dependencies: HashMap<String, FinalPredefinedDependency>,
  prebuild_script: Option<PreBuildScript>
}

impl FinalProjectData {
  pub fn new(unclean_project_root: &str, dep_config: &AllPredefinedDependencies) -> Result<FinalProjectData, String> {
    let project_data_result: FinalProjectData = Self::create_new(unclean_project_root, false, dep_config)?;

    project_data_result.validate_correctness()?;

    return Ok(project_data_result);
  }

  fn create_new(unclean_project_root: &str, is_subproject: bool, all_dep_config: &AllPredefinedDependencies) -> Result<FinalProjectData, String> {
    // NOTE: Subprojects are still considered whole projects, however they are not allowed to specify
    // top level build configuration data. This means that language data, build configs, etc. are not
    // defined in subprojects, and shouldn't be written. Build configuration related data is inherited
    // from the parent project.
    let raw_project: RawProject;
    let project_type: FinalProjectType;

    if is_subproject {
      raw_project = create_subproject_data(&unclean_project_root)?.into();
      project_type = FinalProjectType::Subproject(SubprojectOnlyOptions { })
    } else {
      raw_project = create_project_data(&unclean_project_root)?;
      project_type = FinalProjectType::Full;
    };

    if let Err(err_message) = validate_raw_project(&raw_project) {
      return Err(err_message);
    }

    let project_include_prefix = raw_project.get_include_prefix();
    let project_root: String = cleaned_path_str(&unclean_project_root).to_string();

    let src_dir = format!("{}/src/{}", &project_root, project_include_prefix);
    let include_dir = format!("{}/include/{}", &project_root, project_include_prefix);
    let template_impls_dir = format!("{}/template-impl/{}", &project_root, project_include_prefix);

    let mut subprojects: HashMap<String, FinalProjectData> = HashMap::new();
    // let mut subprojects: Vec<FinalProjectData> = Vec::new();
    // let mut subproject_names: HashSet<String> = HashSet::new();

    if let Some(dirnames) = raw_project.get_subproject_dirnames() {
      for subproject_dirname in dirnames {
        let full_subproject_dir = format!("{}/subprojects/{}", &project_root, subproject_dirname);

        subprojects.insert(
          subproject_dirname.clone(),
          Self::create_new(&full_subproject_dir, true, all_dep_config)?
        );
      }
    }

    let mut output_items: HashMap<String, CompiledOutputItem> = HashMap::new();

    for (output_name, raw_output_item) in raw_project.get_output() {
      output_items.insert(output_name.to_owned(), CompiledOutputItem::from(raw_output_item)?);
    }

    let mut predefined_dependencies: HashMap<String, FinalPredefinedDependency> = HashMap::new();

    if let Some(pre_deps) = &raw_project.predefined_dependencies {
      for (dep_name, user_given_config) in pre_deps {
        let finalized_dep = FinalPredefinedDependency::new(
          all_dep_config,
          dep_name,
          user_given_config
        )?;

        predefined_dependencies.insert(dep_name.into(), finalized_dep);
      }
    }

    let prebuild_script = resolve_prebuild_script(
      &project_root,
      raw_project.prebuild_config.as_ref().unwrap_or(&PreBuildConfigIn {
        link: None
      })
    )?;

    let mut finalized_project_data = FinalProjectData {
      project_type,
      project_root,
      project: raw_project,
      src_dir,
      include_dir,
      template_impls_dir,
      src_files: Vec::<PathBuf>::new(),
      include_files: Vec::<PathBuf>::new(),
      template_impl_files: Vec::<PathBuf>::new(),
      subprojects,
      output: output_items,
      predefined_dependencies,
      prebuild_script
    };

    match populate_files(Path::new(&finalized_project_data.src_dir), &mut finalized_project_data.src_files) {
      Err(err) => return Err(err.to_string()),
      _ => ()
    }

    match populate_files(Path::new(&finalized_project_data.include_dir), &mut finalized_project_data.include_files) {
      Err(err) => return Err(err.to_string()),
      _ => ()
    }

    match populate_files(Path::new(&finalized_project_data.template_impls_dir), &mut finalized_project_data.template_impl_files) {
      Err(err) => return Err(err.to_string()),
      _ => ()
    }

    return Ok(finalized_project_data);
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
            if !final_dep.has_target_named(lib_name_linking) {
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

  fn validate_correctness(&self) -> Result<(), String> {
    for (_, subproject) in &self.subprojects {
      subproject.validate_correctness()?;
    }

    for (output_name, output_item) in &self.output {
      self.ensure_links_are_valid(
        output_name,
        output_item.get_links(),
        false
      )?
    }

    if let Some(existing_script) = &self.prebuild_script {
      match existing_script {
        PreBuildScript::Exe(script_exe_config) => {
          self.ensure_links_are_valid(
            &format!("{}'s pre-build script", self.get_project_name()),
            script_exe_config.get_links(),
            true
          )?;
        },
        PreBuildScript::Python(_) => return Err(format!("Python pre-build scripts are not supported yet."))
      }
    }

    Ok(())
  }

  pub fn has_library_output_named(&self, lib_name: &str) -> bool {
    return match self.get_outputs().get(lib_name) {
      Some(output_item) => output_item.is_library_type(),
      None => false
    }
  }

  pub fn has_prebuild_script(&self) -> bool {
    self.prebuild_script.is_some()
  }

  pub fn has_subprojects(&self) -> bool {
    !self.subprojects.is_empty()
  }

  pub fn has_predefined_dependencies(&self) -> bool {
    self.predefined_dependencies.len() > 0
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

  pub fn get_include_prefix(&self) -> &str {
    return self.project.get_include_prefix();
  }

  pub fn get_project_name(&self) -> &str {
    return self.project.get_name();
  }

  pub fn get_raw_project(&self) -> &RawProject {
    return &self.project;
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
    self.project.get_build_configs()
  }

  pub fn get_default_build_config(&self) -> &BuildType {
    self.project.get_default_build_config()
  }

  pub fn get_language_info(&self) -> &LanguageMap {
    self.project.get_langauge_info()
  }

  pub fn get_global_defines(&self) -> &Option<HashSet<String>> {
    self.project.get_global_defines()
  }
  
  pub fn get_subprojects(&self) -> &HashMap<String, FinalProjectData> {
    &self.subprojects
  }

  pub fn get_project_type(&self) -> &FinalProjectType {
    &self.project_type
  }

  pub fn get_predefined_dependencies(&self) -> &HashMap<String, FinalPredefinedDependency> {
    &self.predefined_dependencies
  }
}

