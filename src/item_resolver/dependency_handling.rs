use std::{collections::HashMap, os::raw, path::{Path, PathBuf}, rc::Rc};
use crate::{data_types::raw_types::RawProject, logger::exit_error_log};

enum CodeFileType {
  Header,
  Source,
  TemplateImpl
}

fn cleaned_path_str(file_path: &str) -> String {
  return path_clean::clean(&file_path.replace("\\", "/"));
}

fn cleaned_pathbuf(file_path: PathBuf) -> PathBuf {
  let replaced_path: String = cleaned_path_str(file_path.to_str().unwrap());
  return PathBuf::from(replaced_path);
}

fn cleaned_path(file_path: &Path) -> PathBuf {
  let replaced_path: String = cleaned_path_str(file_path.to_str().unwrap());
  return PathBuf::from(replaced_path);
}


fn determine_file_type(file_name: &str) -> Result<CodeFileType, String> {
  return if let Some(extension) = Path::new(file_name).extension() {
    match extension.to_str().unwrap() {
      "c" | "cpp" | "cxx" | "c++" => Ok(CodeFileType::Source),
      "h" | "hpp" | "hxx" | "h++" => Ok(CodeFileType::Header),
      "t" | "tpp" | "txx" | "t++" => Ok(CodeFileType::TemplateImpl),
      ext => Err(format!("Invalid extension '{}' for file '{}'", ext, file_name))
    }
  }
  else {
    Err(format!("File '{}' is missing a mandatory extension.", file_name))
  }
}

struct DependencyFileNode {
  file_name: String,
  file_type: CodeFileType,
  dependencies: Vec<Rc<DependencyFileNode>>
}

pub struct DependencyGraph {
  project_root: String,
  entry_files: Vec<Rc<DependencyFileNode>>,
  src_dir: String,
  include_dir: String,
  template_impls_dir: String,
  headers: HashMap<String, Rc<DependencyFileNode>>,
  sources: HashMap<String, Rc<DependencyFileNode>>,
  template_impls: HashMap<String, Rc<DependencyFileNode>>
}

impl DependencyGraph {
  // Entry files are relative to the project root
  pub fn new(
    project_root: &str,
    src_dir: &str,
    include_dir: &str,
    template_impls_dir: &str,
    raw_project_data: &RawProject
  ) -> DependencyGraph {
    let mut graph: DependencyGraph = DependencyGraph {
      project_root: String::from(project_root),
      src_dir: String::from(src_dir),
      include_dir: String::from(include_dir),
      template_impls_dir: String::from(template_impls_dir),
      entry_files: Vec::new(),
      headers: HashMap::new(),
      sources: HashMap::new(),
      template_impls: HashMap::new()
    };

    let entry_filenames: Vec<&str> = raw_project_data.get_output()
      .iter()
      .map(|(_, output_item)| output_item.get_entry_file())
      .collect();

    for entry_file in entry_filenames {
      if let Err(error_message) = graph.add_entry_file(entry_file) {
        exit_error_log(&error_message);
      }
    }

    // TODO: Resolve included and sister files from the entries

    return graph;
  }

  // Nothing should depend on the entry files
  fn add_entry_file(&mut self, relative_file_name: &str) -> Result<Rc<DependencyFileNode>, String> {
    let file_type: CodeFileType = determine_file_type(relative_file_name)?;

    let full_entry_path: PathBuf = cleaned_pathbuf(
      PathBuf::from(&self.project_root)
        .join(relative_file_name)
    );

    println!("Entry path: {}", full_entry_path.to_str().unwrap());

    return if full_entry_path.is_file() {
      let new_entry_file_node: Rc<DependencyFileNode> = Rc::new(DependencyFileNode {
        file_name: String::from(relative_file_name),
        file_type,
        dependencies: Vec::new()
      });

      let cloned_entry = Rc::clone(&new_entry_file_node);

      self.entry_files.push(new_entry_file_node);
      Ok(cloned_entry)
    }
    else {
      Err(format!("Entry file {} does not exist.", relative_file_name))
    }
  }
}