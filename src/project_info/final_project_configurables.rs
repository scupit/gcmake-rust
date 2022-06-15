use std::collections::{HashMap, HashSet};

use super::{raw_data_in::{CompiledItemType, RawCompiledItem, BuildConfigMap, TargetSpecificBuildType, TargetBuildConfigMap, LinkSection}, helpers::{get_link_info, retrieve_file_type, RetrievedCodeFileType}};

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

#[derive(Clone)]
pub enum LinkMode {
  Public,
  Private,
  Interface
}

impl LinkMode {
  pub fn to_str(&self) -> &str {
    match self {
      Self::Public => "public",
      Self::Private => "private",
      Self::Interface => "interface",
    }
  }
}

type LinkMap = HashMap<String, Vec<String>>;

pub struct LinkView<'a> {
  public_links: Option<&'a Vec<String>>,
  interface_links: Option<&'a Vec<String>>,
  private_links: Option<&'a Vec<String>>
}

impl<'a> LinkView<'a> {
  pub fn has_public_links(&self) -> bool {
    return self.public_links.is_some();
  }

  pub fn has_private_links(&self) -> bool {
    return self.private_links.is_some();
  }

  pub fn has_interface_links(&self) -> bool {
    return self.interface_links.is_some();
  }

  pub fn iter_by_link_mode(
    &self,
    wanted_link_modes: impl IntoIterator<Item=LinkMode>
  ) -> impl Iterator<Item=&str> {
    return wanted_link_modes.into_iter()
      .map(move |mode| match mode {
        LinkMode::Public => &self.public_links,
        LinkMode::Private => &self.private_links,
        LinkMode::Interface => &self.interface_links
      })
      .filter(|maybe_link_map| maybe_link_map.is_some())
      .map(|link_map| link_map.unwrap().iter())
      .flatten()
      .map(|string_ref| string_ref.as_str())
  }

  pub fn iter_all(&self) -> impl Iterator<Item=&str> {
    return self.iter_by_link_mode([
      LinkMode::Public,
      LinkMode::Private,
      LinkMode::Interface
    ]);
  }
}

pub struct OutputItemLinks {
  cmake_public: LinkMap,
  cmake_interface: LinkMap,
  cmake_private: LinkMap
}

impl OutputItemLinks {
  pub fn new_empty() -> Self {
    Self {
      cmake_public: LinkMap::new(),
      cmake_private: LinkMap::new(),
      cmake_interface: LinkMap::new()
    }
  }

  pub fn has_links_for_project(
    &self,
    container_project_name: impl AsRef<str>
  ) -> bool {
    return self.iter_link_maps()
      .any(|(link_map, _)| link_map.contains_key(container_project_name.as_ref()))
  }

  pub fn get(&self, container_project_name_val: impl AsRef<str>) -> Option<LinkView> {
    let container_project_name = container_project_name_val.as_ref();

    return if self.has_links_for_project(container_project_name) {
      Some(LinkView {
        public_links: self.cmake_public.get(container_project_name),
        private_links: self.cmake_private.get(container_project_name),
        interface_links: self.cmake_interface.get(container_project_name)
      })
    }
    else {
      None
    }
  }

  pub fn iter_link_maps(&self) -> impl Iterator<Item=(&LinkMap, LinkMode)> {
    return vec![
      (&self.cmake_public, LinkMode::Public),
      (&self.cmake_private, LinkMode::Private),
      (&self.cmake_interface, LinkMode::Interface)
    ].into_iter();
  }

  pub fn all_projects_linked(&self) -> HashSet<String> {
    let mut key_set: HashSet<String> = HashSet::new();

    for (link_map, _) in self.iter_link_maps() {
      key_set.extend(link_map.keys().map(|k| k.to_string()));
    }

    return key_set;
  }
}

pub struct CompiledOutputItem {
  pub output_type: CompiledItemType,
  pub entry_file: String,
  pub links: OutputItemLinks,
  pub build_config: Option<TargetBuildConfigMap>
}

impl CompiledOutputItem {
  pub fn make_link_map(
    output_name: &str,
    output_type: &CompiledItemType,
    raw_links: &LinkSection
  ) -> Result<OutputItemLinks, String> {
    let mut output_links = OutputItemLinks::new_empty();

    match output_type {
      CompiledItemType::Executable => match raw_links {
        LinkSection::PublicPrivateCategorized {..} => {
          return Err(format!(
            "Links given to an executable should not be categorized as public: or private:, however the links provided to this executable are categorized. Please remove the 'public:' and/or 'private:' keys."
          ));
        },
        LinkSection::Uncategorized(link_strings) => {
          combine_links_into(
            link_strings,
            &mut output_links.cmake_private
          )?;
        }
      },
      CompiledItemType::HeaderOnlyLib => match raw_links {
        LinkSection::PublicPrivateCategorized {..} => {
          return Err(format!(
            "Links given to header-only library should not be categorized as public: or private:, however the links provided to this header-only library are categorized. Please remove the 'public:' and/or 'private:' keys."
          ));
        }
        LinkSection::Uncategorized(link_strings) => {
          combine_links_into(
            link_strings,
            &mut output_links.cmake_interface
          )?;
        }
      },
      compiled_lib => match raw_links {
        LinkSection::PublicPrivateCategorized { public , private } => {
          if let Some(public_links) = public {
            combine_links_into(
              public_links,
              &mut output_links.cmake_public
            )?;
          }

          if let Some(private_links) = private {
            combine_links_into(
              private_links,
              &mut output_links.cmake_private
            )?;
          }
        },
        LinkSection::Uncategorized(_) => {
          return Err(format!(
            "Links given to a compiled library should be categorized into public: and/or private: lists. However, the links given to output were provided as a single list. See the docs for information on categorizing compiled library links."
          ));
        }
      }
    }

    let mut already_used: HashMap<String, LinkMode> = HashMap::new();

    for ref container_project in output_links.all_projects_linked() {
      for (link_map, ref map_link_mode) in output_links.iter_link_maps() {
        if let Some(linked_libs) = link_map.get(container_project) {
          for lib_name in linked_libs {
            match already_used.get(lib_name) {
              Some(existing_link_mode) => {
                return Err(format!(
                  "Library {}::{} is linked to '{}' in both {} and {} categories. Libraries should only be linked to an item from a single inheritance category. Make sure the library is listed in either public: or private: lists, but not both.",
                  container_project,
                  lib_name,
                  output_name,
                  existing_link_mode.to_str(),
                  map_link_mode.to_str()
                ));
              },
              None => {
                already_used.insert(lib_name.clone(), map_link_mode.clone());
              }
            }
          }
        }
      }
    }

    return Ok(output_links);
  }

  pub fn from(output_name: &str, raw_output_item: &RawCompiledItem) -> Result<CompiledOutputItem, String> {
    let mut final_output_item = CompiledOutputItem {
      output_type: raw_output_item.output_type,
      entry_file: String::from(&raw_output_item.entry_file),
      links: OutputItemLinks::new_empty(),
      build_config: raw_output_item.build_config.clone()
    };

    if let Some(raw_links) = &raw_output_item.link {
      final_output_item.links = Self::make_link_map(
        output_name,
        final_output_item.get_output_type(),
        raw_links
      )?
    }

    return Ok(final_output_item);
  }

  pub fn get_links(&self) -> &OutputItemLinks {
    &self.links
  }

  pub fn has_links(&self) -> bool {
    return self.links
      .iter_link_maps()
      .any(|(link_map, _)| !link_map.is_empty());
  }

  pub fn get_build_config_map(&self) -> &Option<TargetBuildConfigMap> {
    &self.build_config
  }

  pub fn get_entry_file(&self) -> &str {
    return &self.entry_file;
  }

  pub fn get_output_type(&self) -> &CompiledItemType {
    return &self.output_type;
  }

  pub fn is_header_only_type(&self) -> bool {
    self.output_type == CompiledItemType::HeaderOnlyLib
  }

  pub fn is_library_type(&self) -> bool {
    match self.output_type {
      CompiledItemType::Library
      | CompiledItemType::SharedLib
      | CompiledItemType::StaticLib 
      | CompiledItemType::HeaderOnlyLib => true,
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

fn combine_links_into(
  link_strings: &Vec<String>,
  destination_map: &mut HashMap<String, Vec<String>>
) -> Result<(), String> {
  for link_str in link_strings {
    let LinkInfo {
      from_project_name,
      library_names
    } = get_link_info(link_str)?;

    destination_map
      .entry(from_project_name)
      .and_modify(|lib_list| {
        for lib_name_adding in &library_names {
          if !lib_list.contains(lib_name_adding) {
            lib_list.push(lib_name_adding.clone())
          }
        }
      })
      .or_insert(library_names);
  }

  Ok(())
}