use std::{rc::Rc};

use super::{raw_data_in::{OutputItemType, RawCompiledItem, TargetBuildConfigMap, LinkSection}, final_dependencies::FinalPredefinedDependencyConfig, LinkSpecifier, link_spec_parser::LinkAccessMode};

#[derive(Clone)]
pub enum FinalTestFramework {
  Catch2(Rc<FinalPredefinedDependencyConfig>),
  // GoogleTest(FinalPredefinedDependencyConfig),
  // #[serde(rename = "doctest")]
  // DocTest(FinalPredefinedDependencyConfig),
}

impl FinalTestFramework {
  pub fn unwrap_config(&self) -> Rc<FinalPredefinedDependencyConfig> {
    match self {
      Self::Catch2(ref predep_config) => Rc::clone(predep_config)
    }
  }

  pub fn project_dependency_name(&self) -> &str {
    match self {
      Self::Catch2(_) => "Catch2"
    }
  }

  pub fn main_provided_link_target_name(&self) -> &str {
    match self {
      Self::Catch2(_) => "Catch2WithMain"
    }
  }

  pub fn main_not_provided_link_target_name(&self) -> &str {
    match self {
      Self::Catch2(_) => "Catch2"
    }
  }
}

pub enum FinalProjectType {
  Root,
  Subproject {

  },
  Test {
    framework: FinalTestFramework
  }
}

pub enum PreBuildScript {
  Exe(CompiledOutputItem),
  Python(String)
}

// Ordered from most permissive to least permissive.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum LinkMode {
  Public,
  Interface,
  Private
}

impl LinkMode {
  pub fn to_str(&self) -> &str {
    match self {
      Self::Public => "public",
      Self::Private => "private",
      Self::Interface => "interface",
    }
  }

  pub fn more_permissive(first: Self, second: Self) -> Self {
    return if first > second
      { first }
      else { second }
  }
}

#[derive(Clone)]
pub struct OutputItemLinks {
  pub cmake_public: Vec<LinkSpecifier>,
  pub cmake_interface: Vec<LinkSpecifier>,
  pub cmake_private: Vec<LinkSpecifier>
}

impl OutputItemLinks {
  pub fn new_empty() -> Self {
    Self {
      cmake_public: Vec::new(),
      cmake_private: Vec::new(),
      cmake_interface: Vec::new()
    }
  }
}


pub struct CompiledOutputItem {
  pub output_type: OutputItemType,
  pub entry_file: String,
  pub links: OutputItemLinks,
  pub build_config: Option<TargetBuildConfigMap>,
  pub requires_custom_main: bool
}

impl CompiledOutputItem {
  pub fn make_link_map(
    _output_name: &str,
    output_type: &OutputItemType,
    raw_links: &LinkSection
  ) -> Result<OutputItemLinks, String> {
    let mut output_links = OutputItemLinks::new_empty();

    match output_type {
      OutputItemType::Executable => match raw_links {
        LinkSection::PublicPrivateCategorized {..} => {
          return Err(format!(
            "Links given to an executable should not be categorized as public: or private:, however the links provided to this executable are categorized. Please remove the 'public:' and/or 'private:' keys."
          ));
        },
        LinkSection::Uncategorized(link_strings) => {
          parse_all_links_into(
            link_strings,
            &mut output_links.cmake_private
          )?;
        }
      },
      OutputItemType::HeaderOnlyLib => match raw_links {
        LinkSection::PublicPrivateCategorized {..} => {
          return Err(format!(
            "Links given to header-only library should not be categorized as public: or private:, however the links provided to this header-only library are categorized. Please remove the 'public:' and/or 'private:' keys."
          ));
        }
        LinkSection::Uncategorized(link_strings) => {
          parse_all_links_into(
            link_strings,
            &mut output_links.cmake_interface
          )?;
        }
      },
      OutputItemType::CompiledLib
        | OutputItemType::SharedLib
        | OutputItemType::StaticLib
      => match raw_links {
        LinkSection::PublicPrivateCategorized { public , private } => {
          if let Some(public_links) = public {
            parse_all_links_into(
              public_links,
              &mut output_links.cmake_public
            )?;
          }

          if let Some(private_links) = private {
            parse_all_links_into(
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

    return Ok(output_links);
  }

  pub fn from(output_name: &str, raw_output_item: &RawCompiledItem) -> Result<CompiledOutputItem, String> {
    let mut final_output_item = CompiledOutputItem {
      output_type: raw_output_item.output_type,
      entry_file: String::from(&raw_output_item.entry_file),
      links: OutputItemLinks::new_empty(),
      build_config: raw_output_item.build_config.clone(),
      requires_custom_main: raw_output_item.requires_custom_main.unwrap_or(false)
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

  pub fn get_build_config_map(&self) -> &Option<TargetBuildConfigMap> {
    &self.build_config
  }

  pub fn get_entry_file(&self) -> &str {
    return &self.entry_file;
  }

  pub fn get_output_type(&self) -> &OutputItemType {
    return &self.output_type;
  }

  pub fn is_header_only_type(&self) -> bool {
    self.output_type == OutputItemType::HeaderOnlyLib
  }

  pub fn is_library_type(&self) -> bool {
    match self.output_type {
      OutputItemType::CompiledLib
      | OutputItemType::SharedLib
      | OutputItemType::StaticLib 
      | OutputItemType::HeaderOnlyLib => true,
      OutputItemType::Executable => false
    }
  }

  pub fn is_executable_type(&self) -> bool {
    match self.output_type {
      OutputItemType::Executable => true,
      _ => false
    }
  }
}

fn parse_all_links_into(
  link_strings: &Vec<String>,
  destination_vec: &mut Vec<LinkSpecifier>
) -> Result<(), String> {
  for link_str in link_strings {
    destination_vec.push(LinkSpecifier::parse_from(link_str, LinkAccessMode::UserFacing)?);
  }
  Ok(())
}
