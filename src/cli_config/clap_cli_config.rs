use clap::{AppSettings, ArgEnum, Subcommand, Args, Parser, ValueEnum};

const SKY: &'static str = "Skylar Cupit";

#[derive(ValueEnum, Clone)]
pub enum CLIProjectOutputTypeIn {
  Exe,
  StaticLib,
  SharedLib,
  HeaderOnly,
  CompiledLib
}

#[derive(Parser)]
#[clap(version = "1.3.3", author = SKY)]
#[clap(setting = AppSettings::ColoredHelp)]
pub struct Opts {
  #[clap(default_value = ".")]
  pub project_root: String,

  #[clap(subcommand)]
  pub subcommand: Option<SubCommandStruct>
}

#[derive(Subcommand)]
pub enum SubCommandStruct {
  /// Generate a new project or subproject
  #[clap(subcommand)]
  New(NewProjectSubcommand),

  /// Generate code files in-tree.
  #[clap()]
  GenFile(CreateFilesCommand),

  /// Subcommand for working with the 'external dependency configuration repository'.
  #[clap(subcommand)]
  DepConfig(DepConfigSubCommand)
}

#[derive(Subcommand)]
#[clap(setting = AppSettings::ColoredHelp)]
pub enum NewProjectSubcommand {
  /// Generate a new toplevel project
  #[clap()]
  RootProject(NewProjectCommand),

  /// Generate a new subproject
  #[clap()]
  Subproject(NewSubprojectCommand),

  /// Generate a new test
  #[clap()]
  Test(NewTestProjectCommand)
}

/// Generate a new toplevel project
#[derive(Args)]
#[clap(setting = AppSettings::ColoredHelp)]
pub struct NewProjectCommand {
  /// Project name, no whitespace
  #[clap(required = true)]
  pub new_project_name: String,

  /// Generate a C project and skip language prompt.
  #[clap(long)]
  pub c: bool,

  /// Generate a C++ project and skip language prompt.
  #[clap(long)]
  pub cpp: bool,

  #[clap(value_enum, short, long, name = "type")]
  pub project_type: Option<CLIProjectOutputTypeIn>,
}

/// Generate a new subproject
#[derive(Args)]
#[clap(setting = AppSettings::ColoredHelp)]
pub struct NewSubprojectCommand {
  /// Project name, no whitespace
  #[clap(required = true)]
  pub new_project_name: String,

  /// Generate a C project and skip language prompt.
  #[clap(long)]
  pub c: bool,

  /// Generate a C++ project and skip language prompt.
  #[clap(long)]
  pub cpp: bool,

  #[clap(value_enum, short, long, name = "type")]
  pub subproject_type: Option<CLIProjectOutputTypeIn>
}

/// Generate a new test.
/// Note that all tests are C++ executable subprojects, since only C++
/// test frameworks are currently supported.
#[derive(Args)]
#[clap(setting = AppSettings::ColoredHelp)]
pub struct NewTestProjectCommand {
  /// Project name, no whitespace
  #[clap(required = true)]
  pub new_project_name: String,
}

#[derive(ArgEnum, Clone, Copy)]
pub enum FileCreationLang {
  C,
  Cpp
}

#[derive(Args)]
#[clap(setting = AppSettings::ColoredHelp)]
pub struct CreateFilesCommand {
  #[clap(arg_enum, required = true)]
  pub language: FileCreationLang,

  /// Name of the generated file relative to any code folder.
  /// Example: Assuming file_types == hs (header and source generated)
  /// and language == cpp,
  /// "SomeClass" turns into "include/<FULL_INCLUDE_PREFIX>/SomeClass.hpp" and "src/<FULL_INCLUDE_PREFIX>/SomeClass.cpp" 
  /// while "nested/SomeClass" turns into "include/<FULL_INCLUDE_PREFIX>/nested/SomeClass.hpp" and
  /// "src/<FULL_INCLUDE_PREFIX>/nested/SomeClass.cpp" 
  #[clap(required = true)]
  pub file_name: String,

  /// Combination of 'h' 's' and 't' (example: hs), where 'h' = Header, 's' = Source,
  /// and 't' = Template implementation
  #[clap(required = false, default_value = "hs")]
  pub file_types: String,

  #[clap(short = 'p')]
  pub use_pragma_guards: bool
}

#[derive(Subcommand)]
#[clap(setting = AppSettings::ColoredHelp)]
pub enum DepConfigSubCommand {
  /// Update the dependency configuration repo. Downloads the repo if it is not already present.
  #[clap()]
  Update(UpdateDependencyConfigsCommand)
}

#[derive(Args)]
#[clap(setting = AppSettings::ColoredHelp)]
pub struct UpdateDependencyConfigsCommand {
  /// Selects the branch to be checked out before pulling changes (or after cloning, if the
  /// repo hasn't been installed yet). If no branch is specified, then the current branch is
  /// updated or the repo is cloned into the 'develop' branch.
  #[clap(long = "to-branch", short = 'b')]
  pub branch: Option<String>,
}
