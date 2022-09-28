use std::{rc::Rc, collections::{HashMap, HashSet}, path::PathBuf};

use super::{raw_data_in::{OutputItemType, RawCompiledItem, TargetBuildConfigMap, LinkSection, BuildConfigCompilerSpecifier, BuildType, TargetSpecificBuildType, RawBuildConfig, BuildTypeOptionMap, BuildConfigMap, RawGlobalPropertyConfig, DefaultCompiledLibType, RawShortcutConfig}, final_dependencies::FinalPredefinedDependencyConfig, LinkSpecifier, parsers::{link_spec_parser::LinkAccessMode, general_parser::ParseSuccess}, SystemSpecifierWrapper, platform_spec_parser::parse_leading_system_spec};

#[derive(Clone)]
pub enum FinalTestFramework {
  Catch2(Rc<FinalPredefinedDependencyConfig>),
  GoogleTest(Rc<FinalPredefinedDependencyConfig>),
  DocTest(Rc<FinalPredefinedDependencyConfig>)
}

impl FinalTestFramework {
  pub fn unwrap_config(&self) -> Rc<FinalPredefinedDependencyConfig> {
    match self {
      Self::Catch2(predep_config) => Rc::clone(predep_config),
      Self::DocTest(predep_config) => Rc::clone(predep_config),
      Self::GoogleTest(predep_config) => Rc::clone(predep_config)
    }
  }

  pub fn project_dependency_name(&self) -> &str {
    match self {
      Self::Catch2(_) => "Catch2",
      Self::DocTest(_) => "doctest",
      Self::GoogleTest(_) => "GoogleTest"
    }
  }

  pub fn main_provided_link_target_name(&self) -> &str {
    match self {
      Self::Catch2(_) => "Catch2WithMain",
      Self::DocTest(_) => "doctest_with_main",
      Self::GoogleTest(_) => "gtest_main"
    }
  }

  pub fn main_not_provided_link_target_name(&self) -> &str {
    match self {
      Self::Catch2(_) => "Catch2",
      Self::DocTest(_) => "doctest",
      Self::GoogleTest(_) => "gtest",
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

pub struct FinalShortcutConfig {
  pub shortcut_name: String
}

impl From<RawShortcutConfig> for FinalShortcutConfig {
  fn from(raw_config: RawShortcutConfig) -> Self {
    Self {
      shortcut_name: raw_config.name
    }
  }
}

pub struct FinalInstallerConfig {
  pub title: String,
  pub description: String,
  pub name_prefix: String,
  pub shortcuts: HashMap<String, FinalShortcutConfig>
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
  // NOTE: This is a relative path which references a file RELATIVE TO THE ROOT PROJECT'S ROOT DIRECTORY.
  // That directory is not always the same as the project which directly contains the output item.
  pub windows_icon_relative_to_root_project: Option<PathBuf>,
  pub build_config: Option<FinalTargetBuildConfigMap>,
  pub requires_custom_main: bool
}

impl CompiledOutputItem {
  pub fn export_macro_header_include_path(
    full_include_prefix: &str,
    target_name: &str
  ) -> String {
    return format!(
      "{}/{}_export.h",
      full_include_prefix,
      target_name
    );
  }

  pub fn str_export_macro(target_name: &str) -> String {
    return format!("{}_EXPORT", target_name)
      .to_uppercase()
      .replace(" ", "_")
      .replace("-", "_");
  }

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
      windows_icon_relative_to_root_project: raw_output_item.windows_icon.clone()
        .map(PathBuf::from),
      build_config: make_final_target_build_config(raw_output_item.build_config.as_ref())?,
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

  pub fn get_build_config_map(&self) -> &Option<FinalTargetBuildConfigMap> {
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

  pub fn is_compiled_library_type(&self) -> bool {
    match self.output_type {
      OutputItemType::CompiledLib
      | OutputItemType::SharedLib
      | OutputItemType::StaticLib => true,
      _ => false
    }
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

pub struct CompilerDefine {
  pub system_spec: SystemSpecifierWrapper,
  pub def_string: String
}

impl CompilerDefine {
  pub fn new(define_string: &str) -> Result<Self, String> {
    return match parse_leading_system_spec(define_string)? {
      Some(ParseSuccess { value, rest }) => {
        Ok(Self {
          system_spec: value,
          def_string: rest.to_string()
        })
      },
      None => {
        Ok(Self {
          system_spec: SystemSpecifierWrapper::default_include_all(),
          def_string: define_string.to_string()
        })
      }
    }
  }

  pub fn make_list_from_maybe(maybe_def_list: Option<&Vec<String>>) -> Result<Vec<Self>, String> {
    return match maybe_def_list {
      Some(def_list) => Self::make_list(def_list),
      None => Ok(Vec::new())
    }
  }

  pub fn make_list(def_list: &Vec<String>) -> Result<Vec<Self>, String> {
    def_list.iter()
      .map(|single_def| Self::new(single_def))
      .collect()
  }
}

pub struct CompilerFlag {
  pub system_spec: SystemSpecifierWrapper,
  pub flag_string: String
}

impl CompilerFlag {
  pub fn new(flag_str: &str) -> Result<Self, String> {
    return match parse_leading_system_spec(flag_str)? {
      Some(ParseSuccess { value, rest }) => {
        Ok(Self {
          system_spec: value,
          flag_string: rest.to_string()
        })
      },
      None => {
        Ok(Self {
          system_spec: SystemSpecifierWrapper::default_include_all(),
          flag_string: flag_str.to_string()
        })
      }
    }
  }

  pub fn make_list_from_maybe(maybe_flag_list: Option<&Vec<String>>) -> Result<Vec<Self>, String> {
    return match maybe_flag_list {
      Some(flag_list) => Self::make_list(flag_list),
      None => Ok(Vec::new())
    }
  }

  pub fn make_list(flag_list: &Vec<String>) -> Result<Vec<Self>, String> {
    flag_list.iter()
      .map(|single_flag| Self::new(single_flag))
      .collect()
  }
}

pub type LinkerFlag = CompilerFlag;

pub struct FinalBuildConfig {
  pub compiler_flags: Vec<CompilerFlag>,
  pub linker_flags: Vec<LinkerFlag>,
  pub defines: Vec<CompilerDefine>
}

impl FinalBuildConfig {
  pub fn make_from(raw_build_config: &RawBuildConfig) -> Result<Self, String> {
    Ok(Self {
      compiler_flags: CompilerFlag::make_list_from_maybe(raw_build_config.compiler_flags.as_ref())?,
      linker_flags: LinkerFlag::make_list_from_maybe(raw_build_config.linker_flags.as_ref())?,
      defines: CompilerDefine::make_list_from_maybe(raw_build_config.defines.as_ref())?
    })
  }

  pub fn has_compiler_flags(&self) -> bool {
    !self.compiler_flags.is_empty()
  }

  pub fn has_linker_flags(&self) -> bool {
    !self.linker_flags.is_empty()
  }

  pub fn has_compiler_defines(&self) -> bool {
    !self.defines.is_empty()
  }
}

pub type FinalBuildTypeOptionMap = HashMap<BuildConfigCompilerSpecifier, FinalBuildConfig>;
pub type FinalBuildConfigMap = HashMap<BuildType, FinalBuildTypeOptionMap>;
pub type FinalTargetBuildConfigMap = HashMap<TargetSpecificBuildType, FinalBuildTypeOptionMap>;

pub fn make_final_build_config_map(raw_build_config_map: &BuildConfigMap) -> Result<FinalBuildConfigMap, String> {
  let mut resulting_map: FinalBuildConfigMap = FinalBuildConfigMap::new();

  for (build_type, raw_build_config_by_compiler) in raw_build_config_map {
    resulting_map.insert(
      build_type.clone(),
      make_final_by_compiler_config_map(raw_build_config_by_compiler)?
    );
  }

  return Ok(resulting_map);
}

pub fn make_final_by_compiler_config_map(raw_by_compiler_map: &BuildTypeOptionMap) -> Result<FinalBuildTypeOptionMap, String> {
  let mut resulting_map: FinalBuildTypeOptionMap = FinalBuildTypeOptionMap::new();

  for (compiler_spec, raw_build_config) in raw_by_compiler_map {
    resulting_map.insert(
      compiler_spec.clone(),
      FinalBuildConfig::make_from(raw_build_config)?
    );
  }

  return Ok(resulting_map);
}

pub fn make_final_target_build_config(raw_build_config: Option<&TargetBuildConfigMap>) -> Result<Option<FinalTargetBuildConfigMap>, String> {
  return match raw_build_config {
    None => Ok(None),
    Some(config_map) => {
      let mut resulting_map: FinalTargetBuildConfigMap = FinalTargetBuildConfigMap::new();

      for (target_build_type, by_compiler_config) in config_map {
        resulting_map.insert(
          target_build_type.clone(),
          make_final_by_compiler_config_map(by_compiler_config)?
        );
      }

      Ok(Some(resulting_map))
    }
  }
}

pub struct FinalGlobalProperties {
  pub ipo_enabled_by_default: bool,
  pub default_compiled_lib_type: DefaultCompiledLibType
}

impl FinalGlobalProperties {
  pub fn from_raw(raw_global_properties: &RawGlobalPropertyConfig) -> Self {
    let final_property_config: Self = Self {
      ipo_enabled_by_default: raw_global_properties.ipo_enabled_by_default.clone().unwrap_or(false),
      default_compiled_lib_type: raw_global_properties.default_compiled_lib_type.clone()
        .unwrap_or(DefaultCompiledLibType::Shared)
    };

    return final_property_config;
  }
}
