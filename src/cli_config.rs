use clap::{AppSettings, Clap, ArgEnum};

const SKY: &'static str = "Skylar Cupit";

#[derive(Clap)]
#[clap(version = "1.3.2", author = SKY)]
#[clap(setting = AppSettings::ColoredHelp)]
pub struct Opts {
  #[clap(default_value = ".")]
  pub project_root: String,

  #[clap(subcommand)]
  pub subcommand: Option<SubCommand>
}

#[derive(Clap)]
pub enum SubCommand {
    /// Generate a new project or subproject
    #[clap()]
    New(NewProjectCommand),

    /// Generate code files in-tree.
    #[clap()]
    GenFile(CreateFilesCommand),

    /// Subcommand for working with the 'external dependency configuration repository'.
    #[clap(subcommand)]
    DepConfig(DepConfigSubCommand)
}

/// Generate a new project
#[derive(Clap)]
#[clap(setting = AppSettings::ColoredHelp)]
pub struct NewProjectCommand {
    #[clap(long)]
    pub subproject: bool,

    /// Project name, no whitespace
    #[clap(required = true)]
    pub new_project_name: String,

    /// Generate a C project and skip language prompt.
    #[clap(long)]
    pub c: bool,

    /// Generate a C++ project and skip language prompt.
    #[clap(long)]
    pub cpp: bool,

    #[clap(long)]
    pub static_lib: bool,
    #[clap(long)]
    pub shared_lib: bool,
    #[clap(long)]
    pub library: bool,
    #[clap(long)]
    pub executable: bool
}

#[derive(ArgEnum, Clone, Copy)]
pub enum FileCreationLang {
  C,
  Cpp
}

#[derive(Clap)]
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

#[derive(Clap)]
#[clap(setting = AppSettings::ColoredHelp)]
pub enum DepConfigSubCommand {
  /// Update the dependency configuration repo. Downloads the repo if it is not already present.
  #[clap()]
  Update(UpdateDependencyConfigsCommand)
}

#[derive(Clap)]
#[clap(setting = AppSettings::ColoredHelp)]
pub struct UpdateDependencyConfigsCommand {
  /// Selects the branch to be checked out before pulling changes (or after cloning, if the
  /// repo hasn't been installed yet). If no branch is specified, then the current branch is
  /// updated or the repo is cloned into the 'develop' branch.
  #[clap(long)]
  pub branch: Option<String>,
}
