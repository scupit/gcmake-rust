mod path_manipulation;
use std::{borrow::Borrow, collections::{HashMap, HashSet}, fs::{self, DirEntry, read_dir}, io::{self}, os::windows::raw, path::{Path, PathBuf}};

use crate::{data_types::raw_types::*, item_resolver::path_manipulation::cleaned_path_str};

use self::path_manipulation::{cleaned_path, cleaned_pathbuf};

fn yaml_names_from_dir(project_root: &str) -> Vec<PathBuf> {
  let cmake_data_path: PathBuf = Path::new(project_root)
    .join("cmake_data");

  return vec![
    cmake_data_path.with_extension("yaml"), // ...../cmake_data.yaml
    cmake_data_path.with_extension("yml") // ...../cmake_data.yml
  ];
}

fn create_project_data(project_root: &str) -> Result<RawProject, String> {
  for possible_cmake_data_file in yaml_names_from_dir(project_root) {
    if let io::Result::Ok(cmake_data_yaml_string) = fs::read_to_string(possible_cmake_data_file) {

      return match serde_yaml::from_str::<RawProject>(&cmake_data_yaml_string) {
        Ok(serialized_project) => Ok(serialized_project),
        Err(error) => Err(error.to_string())
      }
    }
  }

  return Err(format!("Unable to find a cmake_data.yaml or cmake_data.yml file in {}", project_root));
}

fn create_subproject_data(project_root: &str) -> Result<RawSubproject, String> {
  for possible_cmake_data_file in yaml_names_from_dir(project_root) {
    if let io::Result::Ok(cmake_data_yaml_string) = fs::read_to_string(possible_cmake_data_file) {

      return match serde_yaml::from_str::<RawSubproject>(&cmake_data_yaml_string) {
        Ok(serialized_project) => Ok(serialized_project),
        Err(error) => Err(error.to_string())
      }
    }
  }

  return Err(format!("Unable to find a cmake_data.yaml or cmake_data.yml file in {}", project_root));
}

fn populate_files(dir: &Path, file_list: &mut Vec<PathBuf>) -> io::Result<()> {
  if dir.is_dir() {
    for dirent in fs::read_dir(dir)? {
      let path = dirent?.path();
      if path.is_dir() {
        populate_files(&path, file_list)?;
      }
      else {
        file_list.push(cleaned_pathbuf(path));
      }
    }
  }
  Ok(())
}

pub struct SubprojectOnlyOptions {
  // TODO: Add subproject only options (such as optional_build)
}

pub enum FinalProjectType {
  Full,
  Subproject(SubprojectOnlyOptions)
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
  subproject_names: HashSet<String>,
  subprojects: Vec<FinalProjectData>
}

impl FinalProjectData {
  pub fn new(unclean_project_root: &str) -> Result<FinalProjectData, String> {
    Self::create_new(unclean_project_root, false)
  }

  fn create_new(unclean_project_root: &str, is_subproject: bool) -> Result<FinalProjectData, String> {
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


    let project_include_prefix = raw_project.get_include_prefix();
    let project_root: String = cleaned_path_str(&unclean_project_root).to_string();

    let src_dir = format!("{}/src/{}", &project_root, project_include_prefix);
    let include_dir = format!("{}/include/{}", &project_root, project_include_prefix);
    let template_impls_dir = format!("{}/template-impl/{}", &project_root, project_include_prefix);

    let mut subprojects: Vec<FinalProjectData> = Vec::new();
    let mut subproject_names: HashSet<String> = HashSet::new();

    if let Some(dirnames) = raw_project.get_subproject_dirnames() {
      for subproject_dirname in dirnames {
        subproject_names.insert(subproject_dirname.clone());

        let full_subproject_dir = format!("{}/subprojects/{}", &project_root, subproject_dirname);
        subprojects.push(Self::create_new(&full_subproject_dir, true)?);
      }
    }

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
      subproject_names,
      subprojects
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

  pub fn has_subprojects(&self) -> bool {
    !self.subprojects.is_empty()
  }

  pub fn get_subproject_names(&self) -> &HashSet<String> {
    &self.subproject_names
  }

  pub fn get_outputs(&self) -> &HashMap<String, RawCompiledItem> {
    self.project.get_output()
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

  pub fn get_global_defines(&self) -> &HashSet<String> {
    self.project.get_global_defines()
  }
  
  pub fn get_subprojects(&self) -> &Vec<FinalProjectData> {
    &self.subprojects
  }

  pub fn get_project_type(&self) -> &FinalProjectType {
    &self.project_type
  }
}
