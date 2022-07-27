use crate::{project_generator::configuration::{MainFileLanguage, CreationProjectOutputType, OutputLibType}};

use self::clap_cli_config::{CLIProjectOutputTypeIn, NewProjectSubcommand};
pub mod clap_cli_config;

pub enum CLIProjectTypeGenerating {
  RootProject,
  Subproject,
  Test
}

pub struct CLIProjectGenerationInfo {
  pub language: Option<MainFileLanguage>,
  pub project_name: String,
  pub project_type: CLIProjectTypeGenerating,
  pub project_output_type: Option<CreationProjectOutputType>
}

impl From<NewProjectSubcommand> for CLIProjectGenerationInfo {
  fn from(command: NewProjectSubcommand) -> Self {
    match command {
      NewProjectSubcommand::RootProject(project_info) => {
        let language: Option<MainFileLanguage> =
          if project_info.c         { Some(MainFileLanguage::C) }
          else if project_info.cpp  { Some(MainFileLanguage::Cpp) }
          else                      { None };

        return CLIProjectGenerationInfo {
          project_name: project_info.new_project_name,
          language,
          project_type: CLIProjectTypeGenerating::RootProject,
          project_output_type: convert_given_project_type(&project_info.project_type)
        }
      },
      NewProjectSubcommand::Subproject(subproject_info) => {
        let language: Option<MainFileLanguage> =
          if subproject_info.c          { Some(MainFileLanguage::C) }
          else if subproject_info.cpp   { Some(MainFileLanguage::Cpp) }
          else                          { None };
        
        return CLIProjectGenerationInfo {
          project_name: subproject_info.new_project_name,
          language,
          project_type: CLIProjectTypeGenerating::Subproject,
          project_output_type: convert_given_project_type(&subproject_info.subproject_type)
        }
      },
      NewProjectSubcommand::Test(test_project_info) =>  {
        return CLIProjectGenerationInfo {
          project_name: test_project_info.new_project_name,
          language: Some(MainFileLanguage::Cpp),
          project_type: CLIProjectTypeGenerating::Test,
          project_output_type: Some(CreationProjectOutputType::Executable)
        }
      }
    }
  }
}

fn convert_given_project_type(given_type: &Option<CLIProjectOutputTypeIn>) -> Option<CreationProjectOutputType> {
  return given_type.as_ref().map(|given_project_type| {
    match given_project_type {
      CLIProjectOutputTypeIn::Exe => CreationProjectOutputType::Executable,
      CLIProjectOutputTypeIn::CompiledLib => CreationProjectOutputType::Library(OutputLibType::ToggleStaticOrShared),
      CLIProjectOutputTypeIn::StaticLib => CreationProjectOutputType::Library(OutputLibType::Static),
      CLIProjectOutputTypeIn::SharedLib => CreationProjectOutputType::Library(OutputLibType::Shared),
      CLIProjectOutputTypeIn::HeaderOnly => CreationProjectOutputType::Library(OutputLibType::HeaderOnly)
    }
  });
}