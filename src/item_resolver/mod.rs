mod path_manipulation;
use std::{borrow::Borrow, collections::{HashMap, HashSet}, fs::{self, DirEntry, read_dir}, io::{self, Result}, path::{Path, PathBuf}};

use crate::{data_types::raw_types::*, item_resolver::path_manipulation::cleaned_path_str};

use self::path_manipulation::{cleaned_path, cleaned_pathbuf};

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


pub struct FinalProjectData {
  project_root: String,
  project: RawProject,
  src_dir: String,
  include_dir: String,
  template_impls_dir: String,
  pub src_files: Vec<PathBuf>,
  pub include_files: Vec<PathBuf>,
  pub template_impl_files: Vec<PathBuf>
}

impl FinalProjectData {
  pub fn new(
    unclean_project_root: String,
    project: RawProject
  ) -> FinalProjectData {
    let project_include_prefix = project.get_include_prefix();
    let project_root: String = cleaned_path_str(&unclean_project_root).to_string();

    let src_dir = format!("{}/src/{}", project_root, project_include_prefix);
    let include_dir = format!("{}/include/{}", project_root, project_include_prefix);
    let template_impls_dir = format!("{}/template-impl/{}", project_root, project_include_prefix);

    let mut finalized_project_data = FinalProjectData {
      project_root,
      project,
      src_dir,
      include_dir,
      template_impls_dir,
      src_files: Vec::<PathBuf>::new(),
      include_files: Vec::<PathBuf>::new(),
      template_impl_files: Vec::<PathBuf>::new()
    };

    populate_files(Path::new(&finalized_project_data.src_dir), &mut finalized_project_data.src_files);
    populate_files(Path::new(&finalized_project_data.include_dir), &mut finalized_project_data.include_files);
    populate_files(Path::new(&finalized_project_data.template_impls_dir), &mut finalized_project_data.template_impl_files);

    return finalized_project_data;
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

  pub fn get_languages(&self) -> HashSet<&str> {
    self.project.languages
      .iter()
      .map(|str| str as &str)
      .collect()
  }
}
