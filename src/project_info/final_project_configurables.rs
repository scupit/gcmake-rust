use std::{rc::Rc, collections::{HashMap, BTreeSet, BTreeMap}, path::{PathBuf, Path}};
use std::hash::{ Hash, Hasher };

use super::{raw_data_in::{OutputItemType, RawCompiledItem, TargetBuildConfigMap, LinkSection, BuildConfigCompilerSpecifier, BuildType, TargetSpecificBuildType, RawBuildConfig, BuildTypeOptionMap, BuildConfigMap, RawGlobalPropertyConfig, DefaultCompiledLibType, RawShortcutConfig, RawFeatureConfig}, final_dependencies::FinalPredefinedDependencyConfig, LinkSpecifier, parsers::{link_spec_parser::LinkAccessMode, general_parser::ParseSuccess}, SystemSpecifierWrapper, platform_spec_parser::parse_leading_system_spec, helpers::{RetrievedCodeFileType, code_file_type}, path_manipulation::{cleaned_pathbuf}, final_project_data::CppFileGrammar};

#[derive(Clone)]
pub struct CodeFileInfo {
  pub file_path: PathBuf,
  pub file_type: RetrievedCodeFileType,
  pub is_generated: bool
}

impl CodeFileInfo {
  pub fn from_path(
    given_path: impl AsRef<Path>,
    is_generated: bool
  ) -> Self {
    let path_ref: &Path = given_path.as_ref();

    Self {
      file_path: cleaned_pathbuf(path_ref),
      file_type: code_file_type(path_ref),
      is_generated
    }
  }

  pub fn uses_cpp2_grammar(&self) -> bool {
    match &self.file_type {
      RetrievedCodeFileType::Source { used_grammar } => match used_grammar {
        CppFileGrammar::Cpp1 => false,
        CppFileGrammar::Cpp2 => true,
      },
      _ => false
    }
  }

  pub fn get_file_path(&self) -> &Path {
    self.file_path.as_path()
  }

  pub fn code_file_type(&self) -> RetrievedCodeFileType {
    self.file_type.clone()
  }
}

impl Hash for CodeFileInfo {
  fn hash<H: Hasher>(&self, state: &mut H) {
    self.file_path.hash(state);
  }
}

impl PartialEq for CodeFileInfo {
  fn eq(&self, other: &Self) -> bool {
    self.file_path == other.file_path
  }
}

impl Eq for CodeFileInfo { }

impl AsRef<CodeFileInfo> for CodeFileInfo {
  fn as_ref(&self) -> &CodeFileInfo {
    self
  }
}

impl PartialOrd for CodeFileInfo {
  fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
    self.file_path.partial_cmp(&other.file_path) 
  }
}

impl Ord for CodeFileInfo {
  fn cmp(&self, other: &Self) -> std::cmp::Ordering {
    self.partial_cmp(other).unwrap()
  }
}

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
      Self::Catch2(_) => "catch2",
      Self::DocTest(_) => "doctest",
      Self::GoogleTest(_) => "googletest"
    }
  }

  pub fn main_provided_link_target_name(&self) -> &str {
    match self {
      Self::Catch2(_) => "with_main",
      Self::DocTest(_) => "with_main",
      Self::GoogleTest(_) => "with_main"
    }
  }

  pub fn main_not_provided_link_target_name(&self) -> &str {
    match self {
      Self::Catch2(_) => "without_main",
      Self::DocTest(_) => "without_main",
      Self::GoogleTest(_) => "without_main",
    }
  }
}

pub enum FinalDocGeneratorName {
  Doxygen,
  Sphinx
}

impl FinalDocGeneratorName {
  pub fn to_str(&self) -> &str {
    match self {
      Self::Doxygen => "Doxygen",
      Self::Sphinx => "Sphinx"
    }
  }
}

pub struct FinalDocumentationInfo {
  pub generator: FinalDocGeneratorName,
  pub headers_only: bool
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

pub struct PreBuildScript {
  pub type_config: PreBuildScriptType,
  pub generated_code: BTreeSet<CodeFileInfo>
}

impl PreBuildScript {
  pub fn get_type(&self) -> &PreBuildScriptType {
    return &self.type_config;
  }

  fn generated_files_by_type(&self, file_type: RetrievedCodeFileType) -> BTreeSet<&CodeFileInfo> {
    return self.generated_code
      .iter()
      .filter(|code_file|  code_file.file_type.is_same_general_type_as(&file_type))
      .collect();
  }

  pub fn generated_sources(&self) -> BTreeSet<&CodeFileInfo> {
    self.generated_files_by_type(RetrievedCodeFileType::Source {
      // The grammar value is ignored for the check.
      used_grammar: CppFileGrammar::Cpp1
    })
  }

  pub fn generated_headers(&self) -> BTreeSet<&CodeFileInfo> {
    self.generated_files_by_type(RetrievedCodeFileType::Header)
  }

  pub fn generated_template_impls(&self) -> BTreeSet<&CodeFileInfo> {
    self.generated_files_by_type(RetrievedCodeFileType::TemplateImpl)
  }
}

pub enum PreBuildScriptType {
  Exe(CompiledOutputItem),
  Python(PathBuf)
}

#[derive(Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct FinalFeatureEnabler {
  pub dep_name: Option<String>,
  pub feature_name: String
}

impl FinalFeatureEnabler {
  pub fn make_from(feature_spec: impl AsRef<str>) -> Result<Self, String> {
    let sections: Vec<&str> = feature_spec.as_ref().split('/').collect();

    if sections.len() > 2 {
      return Err(format!(
        "\"{}\" is not in valid feature enabler format.",
        feature_spec.as_ref()
      ));
    }

    let mut sections_iter = sections.iter();

    return if sections.len() == 1 {
      Ok(FinalFeatureEnabler {
        dep_name: None,
        feature_name: sections_iter.nth(0).unwrap().to_string()
      })
    }
    else {
      Ok(FinalFeatureEnabler {
        dep_name: Some(sections_iter.nth(0).unwrap().to_string()),
        feature_name: sections_iter.nth(0).unwrap().to_string(),
      })
    }
  }
}

pub struct FinalFeatureConfig {
  pub is_enabled_by_default: bool,
  pub enables: BTreeSet<FinalFeatureEnabler>
}

impl FinalFeatureConfig {
  pub fn make_from(raw_feature_config: RawFeatureConfig) -> Result<Self, String> {
    let enables_results: Result<BTreeSet<FinalFeatureEnabler>, String> = raw_feature_config.enables.as_ref()
      .map_or(Ok(BTreeSet::new()), |enables_set|
        enables_set.iter()
          .map(FinalFeatureEnabler::make_from)
          .collect()
      );

    return Ok(Self {
      enables: enables_results?,
      is_enabled_by_default: raw_feature_config.default
    });
  }
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

pub struct FileRootGroup {
  pub project_root: PathBuf,
  pub src_root: PathBuf,
  pub header_root: PathBuf
}


pub struct CompiledOutputItem {
  pub output_type: OutputItemType,
  pub entry_file: CodeFileInfo,
  pub links: OutputItemLinks,
  // NOTE: This is a relative path which references a file RELATIVE TO THE ROOT PROJECT'S ROOT DIRECTORY.
  // That directory is not always the same as the project which directly contains the output item.
  pub windows_icon_relative_to_root_project: Option<PathBuf>,
  pub emscripten_html_shell_relative_to_project_root: Option<PathBuf>,
  pub build_config: Option<FinalTargetBuildConfigMap>,
  pub system_specifier: SystemSpecifierWrapper,
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
    raw_links: &LinkSection,
    valid_feature_list: Option<&Vec<&str>>
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
            &mut output_links.cmake_private,
            valid_feature_list
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
            &mut output_links.cmake_interface,
            valid_feature_list
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
              &mut output_links.cmake_public,
              valid_feature_list
            )?;
          }

          if let Some(private_links) = private {
            parse_all_links_into(
              private_links,
              &mut output_links.cmake_private,
              valid_feature_list
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

  fn resolve_full_build_config(
    raw_output_item: &RawCompiledItem,
    valid_feature_list: Option<&Vec<&str>>
  ) -> Result<Option<FinalTargetBuildConfigMap>, String> {
    match raw_output_item.defines.clone() {
      None => {
        return make_final_target_build_config(
          raw_output_item.build_config.as_ref(),
          valid_feature_list
        );
      },
      Some(defines_list) => {
        let mut cloned_build_config: TargetBuildConfigMap = raw_output_item.build_config.clone().unwrap_or(BTreeMap::new());
        let all_configs_compilers_build_config: &mut RawBuildConfig = cloned_build_config
          .entry(TargetSpecificBuildType::AllConfigs)
          .or_insert(BTreeMap::new())
          .entry(BuildConfigCompilerSpecifier::AllCompilers)
          .or_insert(RawBuildConfig {
            compiler_flags: None,
            link_time_flags: None,
            linker_flags: None,
            defines: None
          });

        match &mut all_configs_compilers_build_config.defines {
          Some(existing_defines) => existing_defines.extend(defines_list),
          None => all_configs_compilers_build_config.defines = Some(defines_list)
        };

        return make_final_target_build_config(Some(&cloned_build_config), valid_feature_list);
      }
    }
  }

  // root_directory must be absolute.
  pub fn make_from(
    output_name: &str,
    raw_output_item: &RawCompiledItem,
    maybe_system_specifier: Option<SystemSpecifierWrapper>,
    valid_feature_list: Option<&Vec<&str>>
  ) -> Result<CompiledOutputItem, String> {
    let mut final_output_item = CompiledOutputItem {
      output_type: raw_output_item.output_type,
      // TODO: Constrain entry file to only the project root
      entry_file: CodeFileInfo::from_path(&raw_output_item.entry_file, false),
      links: OutputItemLinks::new_empty(),
      system_specifier: maybe_system_specifier.unwrap_or_default(),
      windows_icon_relative_to_root_project: raw_output_item.windows_icon.clone()
        .clone()
        .map(PathBuf::from),
      emscripten_html_shell_relative_to_project_root: raw_output_item.emscripten_html_shell
        .clone()
        .map(PathBuf::from),
      build_config: Self::resolve_full_build_config(raw_output_item, valid_feature_list)?,
      requires_custom_main: raw_output_item.requires_custom_main.unwrap_or(false)
    };

    if let Some(raw_links) = &raw_output_item.link {
      final_output_item.links = Self::make_link_map(
        output_name,
        final_output_item.get_output_type(),
        raw_links,
        valid_feature_list
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

  pub fn get_entry_file(&self) -> &CodeFileInfo {
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
  destination_vec: &mut Vec<LinkSpecifier>,
  valid_feature_list: Option<&Vec<&str>>
) -> Result<(), String> {
  for link_str in link_strings {
    destination_vec.push(LinkSpecifier::parse_from(link_str, LinkAccessMode::UserFacing, valid_feature_list)?);
  }
  Ok(())
}

pub struct CompilerDefine {
  pub system_spec: SystemSpecifierWrapper,
  pub def_string: String
}

impl CompilerDefine {
  pub fn new(
    define_string: &str,
    valid_feature_list: Option<&Vec<&str>>
  ) -> Result<Self, String> {
    return match parse_leading_system_spec(define_string, valid_feature_list)? {
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

  pub fn make_list_from_maybe(
    maybe_def_list: Option<&Vec<String>>,
    valid_feature_list: Option<&Vec<&str>>
  ) -> Result<Vec<Self>, String> {
    return match maybe_def_list {
      Some(def_list) => Self::make_list(def_list, valid_feature_list),
      None => Ok(Vec::new())
    }
  }

  pub fn make_list(
    def_list: &Vec<String>,
    valid_feature_list: Option<&Vec<&str>>
  ) -> Result<Vec<Self>, String> {
    def_list.iter()
      .map(|single_def| Self::new(single_def, valid_feature_list))
      .collect()
  }
}

pub struct CompilerFlag {
  pub system_spec: SystemSpecifierWrapper,
  pub flag_string: String
}

impl CompilerFlag {
  pub fn new(
    flag_str: &str,
    valid_feature_list: Option<&Vec<&str>>
  ) -> Result<Self, String> {
    return match parse_leading_system_spec(flag_str, valid_feature_list)? {
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

  pub fn make_list_from_maybe(
    maybe_flag_list: Option<&Vec<String>>,
    valid_feature_list: Option<&Vec<&str>>
  ) -> Result<Vec<Self>, String> {
    return match maybe_flag_list {
      Some(flag_list) => Self::make_list(flag_list, valid_feature_list),
      None => Ok(Vec::new())
    }
  }

  pub fn make_list(
    flag_list: &Vec<String>,
    valid_feature_list: Option<&Vec<&str>>
  ) -> Result<Vec<Self>, String> {
    flag_list.iter()
      .map(|single_flag| Self::new(single_flag, valid_feature_list))
      .collect()
  }
}

pub type LinkerFlag = CompilerFlag;

pub struct FinalBuildConfig {
  pub compiler_flags: Vec<CompilerFlag>,
  pub link_time_flags: Vec<CompilerFlag>,
  pub linker_flags: Vec<LinkerFlag>,
  pub defines: Vec<CompilerDefine>
}

impl FinalBuildConfig {
  pub fn make_from(
    raw_build_config: &RawBuildConfig,
    valid_feature_list: Option<&Vec<&str>>
  ) -> Result<Self, String> {
    Ok(Self {
      compiler_flags: CompilerFlag::make_list_from_maybe(
        raw_build_config.compiler_flags.as_ref(),
        valid_feature_list
      )?,
      link_time_flags: CompilerFlag::make_list_from_maybe(
        raw_build_config.link_time_flags.as_ref(),
        valid_feature_list
      )?,
      linker_flags: LinkerFlag::make_list_from_maybe(
        raw_build_config.linker_flags.as_ref(),
        valid_feature_list
      )?,
      defines: CompilerDefine::make_list_from_maybe(
        raw_build_config.defines.as_ref(),
        valid_feature_list
      )?
    })
  }

  pub fn has_compiler_flags(&self) -> bool {
    !self.compiler_flags.is_empty()
  }

  pub fn has_linker_flags(&self) -> bool {
    !self.linker_flags.is_empty()
  }

  pub fn has_link_time_flags(&self) -> bool {
    !self.link_time_flags.is_empty()
  }

  pub fn has_compiler_defines(&self) -> bool {
    !self.defines.is_empty()
  }
}

pub type FinalBuildTypeOptionMap = BTreeMap<BuildConfigCompilerSpecifier, FinalBuildConfig>;
pub type FinalBuildConfigMap = BTreeMap<BuildType, FinalBuildTypeOptionMap>;
pub type FinalTargetBuildConfigMap = BTreeMap<TargetSpecificBuildType, FinalBuildTypeOptionMap>;

pub fn make_final_build_config_map(
  raw_build_config_map: &BuildConfigMap,
  valid_feature_list: Option<&Vec<&str>>
) -> Result<FinalBuildConfigMap, String> {
  let mut resulting_map: FinalBuildConfigMap = FinalBuildConfigMap::new();

  for (build_type, raw_build_config_by_compiler) in raw_build_config_map {
    resulting_map.insert(
      build_type.clone(),
      make_final_by_compiler_config_map(raw_build_config_by_compiler, valid_feature_list)?
    );
  }

  return Ok(resulting_map);
}

pub fn make_final_by_compiler_config_map(
  raw_by_compiler_map: &BuildTypeOptionMap,
  valid_feature_list: Option<&Vec<&str>>
) -> Result<FinalBuildTypeOptionMap, String> {
  let mut resulting_map: FinalBuildTypeOptionMap = FinalBuildTypeOptionMap::new();

  for (compiler_spec, raw_build_config) in raw_by_compiler_map {
    resulting_map.insert(
      compiler_spec.clone(),
      FinalBuildConfig::make_from(raw_build_config, valid_feature_list)?
    );
  }

  return Ok(resulting_map);
}

pub fn make_final_target_build_config(
  raw_build_config: Option<&TargetBuildConfigMap>,
  valid_feature_list: Option<&Vec<&str>>
) -> Result<Option<FinalTargetBuildConfigMap>, String> {
  return match raw_build_config {
    None => Ok(None),
    Some(config_map) => {
      let mut resulting_map: FinalTargetBuildConfigMap = FinalTargetBuildConfigMap::new();

      for (target_build_type, by_compiler_config) in config_map {
        resulting_map.insert(
          target_build_type.clone(),
          make_final_by_compiler_config_map(by_compiler_config, valid_feature_list)?
        );
      }

      Ok(Some(resulting_map))
    }
  }
}

pub struct FinalGlobalProperties {
  pub ipo_enabled_by_default: bool,
  pub default_compiled_lib_type: DefaultCompiledLibType,
  pub are_language_extensions_enabled: bool
}

impl FinalGlobalProperties {
  pub fn from_raw(raw_global_properties: &RawGlobalPropertyConfig) -> Self {
    let final_property_config: Self = Self {
      ipo_enabled_by_default: raw_global_properties.ipo_enabled_by_default.clone().unwrap_or(false),
      default_compiled_lib_type: raw_global_properties.default_compiled_lib_type.clone()
        .unwrap_or(DefaultCompiledLibType::Shared),
      are_language_extensions_enabled: raw_global_properties.are_language_extensions_enabled.unwrap_or(false)
    };

    return final_property_config;
  }
}
