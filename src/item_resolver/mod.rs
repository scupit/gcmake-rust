pub mod path_manipulation;

use std::{ascii::AsciiExt, borrow::Borrow, collections::{HashMap, HashSet}, fs::{self, DirEntry, read_dir}, io::{self}, os::windows::raw, path::{Path, PathBuf}};
use crate::{data_types::raw_types::*, item_resolver::path_manipulation::cleaned_path_str};
use self::path_manipulation::{cleaned_path, cleaned_pathbuf};
use regex::{Captures, Regex};

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

struct LinkInfo {
  from_project_name: String,
  library_names: Vec<String>
}

// Note that index 0 containes the whole capture (string matching)
fn extract_capture_str<'a>(captures: &'a Captures, index: usize) -> &'a str {
  return captures.get(index)
    .unwrap()
    .as_str()
}

// Note that index 0 containes the whole capture (string matching)
fn extract_capture_string(captures: &Captures, index: usize) -> String {
  return extract_capture_str(captures, index).to_owned()
}

fn get_link_info(link_str: &str) -> Result<LinkInfo, String> {
  /* Matches subproject_name::lib_name
    Capture 1: subproject_name
    Capture 2: lib_name
  */
  let single_link_matcher = Regex::new(r"^([a-zA-z0-9_\-.]+)::([a-zA-z0-9_\-.]+)$").unwrap();

  /* 
    Matches subproject_name::{ lib_name }
    Matches subproject_name::{ lib_name, another_lib_name }

    Capture 1: subproject_name
    Capture 2: { lib_name, another_lib_name }
    
    The second capture matches the whole list including brackets.
  */
  let mutli_link_matcher = Regex::new(r"^([a-zA-z0-9_\-.]+)::(\{ ?(?:[a-zA-z0-9_\-.]+, ?)*[a-zA-z0-9_\-.]+ ?\})$").unwrap();


  if let Some(captures) = single_link_matcher.captures(link_str) {
    return Ok(LinkInfo {
      from_project_name: extract_capture_string(&captures, 1),
      library_names: vec![extract_capture_string(&captures, 2)]
    });
  }
  else if let Some(captures) = mutli_link_matcher.captures(link_str) {
    let mut lib_links_list: &str = extract_capture_str(&captures, 2);

    {
      let open_bracket_index: usize = lib_links_list.find('{').unwrap();
      let close_bracket_index: usize = lib_links_list.rfind('}').unwrap();

      lib_links_list = (&lib_links_list[open_bracket_index + 1 .. close_bracket_index]).trim();
    }
    
    return Ok(LinkInfo {
      from_project_name: extract_capture_string(&captures, 1),
      library_names: lib_links_list.split(',')
        .map(|lib_name| lib_name.trim().to_owned())
        .collect()
    });
  }

  return Err(format!("Link specifier \"{}\" is in an invalid format", link_str));
}

pub struct SubprojectOnlyOptions {
  // TODO: Add subproject only options (such as optional_build)
}

pub enum FinalProjectType {
  Full,
  Subproject(SubprojectOnlyOptions)
}

fn validate_output_config(project_data: &RawProject) -> Result<(), String> {
  let mut makes_executable: bool = false;
  let mut makes_library: bool = false;

  for (output_name, output_data) in project_data.get_output() {
    if output_data.is_library_type() {
      if makes_library {
        return Err(format!("Project \"{}\" contains more than one library output, but should only contain one.", project_data.get_name()));
      }

      makes_library = true;

      if makes_executable {
        break;
      }
    }
    else if output_data.is_executable_type() {
      makes_executable = true;

      if makes_library {
        break
      }
    }
  }

  return if makes_executable && makes_library {
    Err(format!("Project \"{}\" should not create both library and executable outputs.", project_data.get_name()))
  }
  else {
    Ok(())
  }
}

fn validate_raw_project(project_data: &RawProject) -> Result<(), String> {
  if let Err(message) = validate_output_config(project_data) {
    return Err(message)
  }

  Ok(())
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
  output: HashMap<String, CompiledOutputItem>
}

impl FinalProjectData {
  pub fn new(unclean_project_root: &str) -> Result<FinalProjectData, String> {
    let project_data_result: FinalProjectData = Self::create_new(unclean_project_root, false)?;

    project_data_result.validate_correctness()?;

    return Ok(project_data_result);
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

        subprojects.insert(subproject_dirname.clone(), Self::create_new(&full_subproject_dir, true)?);
      }
    }

    let mut output_items: HashMap<String, CompiledOutputItem> = HashMap::new();

    for (output_name, raw_output_item) in raw_project.get_output() {
      output_items.insert(output_name.to_owned(), CompiledOutputItem::from(raw_output_item)?);
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
      subprojects,
      output: output_items
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

  fn validate_correctness(&self) -> Result<(), String> {
    for (_, subproject) in &self.subprojects {
      subproject.validate_correctness()?;
    }

    for (output_name, output_item) in &self.output {
      if let Some(link_map) = &output_item.links {
        // Each library linked to an output item should be member of a subproject or dependency
        // project. This loop checks that each of the referenced sub/dependency project names
        // exist and if they do, that the linked libraries from withing those projects exist
        // as well.
        for (project_name_containing_libraries, lib_names_linking) in link_map {
          match self.subprojects.get_key_value(project_name_containing_libraries) {
            // NOTE: matching_subproject_name might be redundant, since it's the same
            // as project_name_containing_libraries.
            Some((matching_subproject_name, matching_subproject)) => {
              for lib_name_linking in lib_names_linking {
                if !matching_subproject.has_library_output_named(lib_name_linking) {
                  return Err(format!(
                    "Output item '{}' in project '{}' tries to link to a nonexistent library '{}' in subproject '{}'.",
                    output_name,
                    self.get_project_name(),
                    lib_name_linking,
                    matching_subproject_name
                  ));
                }
              }
            },
            None => return Err(format!(
              "Output item '{}' in project '{}' tries to link to libraries in a project named '{}', however that project doesn't exist.",
              output_name,
              self.get_project_name(),
              project_name_containing_libraries
            ))
          }
        }
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

  pub fn has_subprojects(&self) -> bool {
    !self.subprojects.is_empty()
  }

  pub fn get_subproject_names(&self) -> HashSet<String> {
    self.subprojects.iter()
      .map(|(subproject_name, _)| subproject_name.to_owned())
      .collect()
  }

  pub fn get_outputs(&self) -> &HashMap<String, CompiledOutputItem> {
    &self.output
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
}

pub struct CompiledOutputItem {
  pub output_type: CompiledItemType,
  pub entry_file: String,
  links: Option<HashMap<String, Vec<String>>>
}

impl CompiledOutputItem {
  pub fn from(raw_output_item: &RawCompiledItem) -> Result<CompiledOutputItem, String> {
    let mut final_output_item = CompiledOutputItem {
      output_type: raw_output_item.output_type,
      entry_file: String::from(&raw_output_item.entry_file),
      links: None
    };

    if let Some(raw_links) = &raw_output_item.link {
      let mut links_by_project: HashMap<String, Vec<String>> = HashMap::new();
      
      for link_str in raw_links {
        match get_link_info(link_str) {
          Ok(LinkInfo { from_project_name, mut library_names }) => {
            if let Some(lib_list) = links_by_project.get_mut(&from_project_name) {
              lib_list.append(&mut library_names)
            }
            else {
              links_by_project.insert(from_project_name, library_names);
            }
          },
          Err(message) => return Err(message)
        }
      }

      final_output_item.links = Some(links_by_project);
    }

    return Ok(final_output_item);
  }

  pub fn get_links(&self) -> &Option<HashMap<String, Vec<String>>> {
    &self.links
  }

  pub fn has_links(&self) -> bool {
    if let Some(links) = &self.links {
      return !links.is_empty();
    }
    return false;
  }

  pub fn get_entry_file(&self) -> &str {
    return &self.entry_file;
  }

  pub fn get_output_type(&self) -> &CompiledItemType {
    return &self.output_type;
  }

  pub fn is_library_type(&self) -> bool {
    match self.output_type {
      CompiledItemType::Library
      | CompiledItemType::SharedLib
      | CompiledItemType::StaticLib => true,
      CompiledItemType::Executable => false
    }
  }

  pub fn is_executable_type(&self) -> bool {
    match self.output_type {
      CompiledItemType::Executable => true,
      _ => false
    }
  }
}
