use std::collections::HashMap;

use super::{raw_data_in::{CompiledItemType, RawCompiledItem}, helpers::get_link_info};

pub struct SubprojectOnlyOptions {
  // TODO: Add subproject only options (such as optional_build)
}

pub enum FinalProjectType {
  Root,
  Subproject(SubprojectOnlyOptions)
}

pub struct LinkInfo {
  pub from_project_name: String,
  pub library_names: Vec<String>
}

pub enum PreBuildScript {
  Exe(CompiledOutputItem),
  Python(String)
}

pub struct CompiledOutputItem {
  pub output_type: CompiledItemType,
  pub entry_file: String,
  pub links: Option<HashMap<String, Vec<String>>>
}

impl CompiledOutputItem {
  pub fn make_link_map(raw_links: &Vec<String>) -> Result<HashMap<String, Vec<String>>, String> {
    let mut links_by_project: HashMap<String, Vec<String>> = HashMap::new();
    
    for link_str in raw_links {
      let LinkInfo { from_project_name, mut library_names } = get_link_info(link_str)?;

      if let Some(lib_list) = links_by_project.get_mut(&from_project_name) {
        lib_list.append(&mut library_names)
      }
      else {
        links_by_project.insert(from_project_name, library_names);
      }
    }

    // final_output_item.links = Some(links_by_project);
    return Ok(links_by_project);
  }
  pub fn from(raw_output_item: &RawCompiledItem) -> Result<CompiledOutputItem, String> {
    let mut final_output_item = CompiledOutputItem {
      output_type: raw_output_item.output_type,
      entry_file: String::from(&raw_output_item.entry_file),
      links: None
    };

    if let Some(raw_links) = &raw_output_item.link {
      final_output_item.links = Some(Self::make_link_map(raw_links)?);
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