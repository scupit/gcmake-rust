use clap::{Subcommand, Args, Parser, ValueEnum};

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
#[command(version = "1.5.7", author = SKY)]
pub struct Opts {
  #[arg(default_value = ".")]
  pub project_root: String,

  #[command(subcommand)]
  pub subcommand: Option<SubCommandStruct>
}

#[derive(Subcommand)]
pub enum SubCommandStruct {
  /// Subcommand for generating new root projects, subprojects, and tests.
  #[command(subcommand)]
  New(NewProjectSubcommand),

  /// Generate code files in-tree.
  GenFile(CreateFilesCommand),

  /// Subcommand for working with the 'external dependency configuration repository'.
  #[command(subcommand)]
  DepConfig(DepConfigSubCommand),

  /// Copy a default file from ~/.gcmake into the project root.
  UseFile(UseFilesCommand),

  /// Select and print information about project outputs and pre-build script.
  TargetInfo(TargetInfoCommand),
  
  /// Select and print information about projects. Dependency print information is limited.
  ProjectInfo(ProjectInfoCommand),

  /// Select and print information about predefined dependencies
  PredepInfo(PredepInfoCommand),

  /// Print information about the GCMake tool itself
  ToolInfo(ToolInfoCommand)
}

#[derive(Subcommand)]
pub enum NewProjectSubcommand {
  /// Generate a new toplevel project
  RootProject(NewRootProjectCommand),

  /// Generate a new subproject
  Subproject(NewSubprojectCommand),

  /// Generate a new test
  Test(NewTestProjectCommand)
}

/// Generate a new toplevel project
#[derive(Args)]
pub struct NewRootProjectCommand {
  /// Project name, no whitespace
  #[arg(required = true)]
  pub new_project_name: String,

  /// Generate a C project and skip language prompt.
  #[arg(long)]
  pub c: bool,

  /// Generate a C++ project and skip language prompt.
  #[arg(long)]
  pub cpp: bool,

  /// Generates a C++ project, but uses .cpp2 files instead of .cpp.
  /// This implies --cpp, so it also skips language prompt.
  #[arg(long)]
  pub cpp2: bool,

  // Specifies the initial project's output type (executable, shared library, etc.).
  #[arg(value_enum, short, long, name = "type")]
  pub project_type: Option<CLIProjectOutputTypeIn>,

  // Omits Emscripten from the list of supported compilers
  #[arg(long)]
  pub no_emscripten: bool
}

/// Generate a new subproject
#[derive(Args)]
pub struct NewSubprojectCommand {
  /// Project name, no whitespace
  #[arg(required = true)]
  pub new_project_name: String,

  /// Generate a C project and skip language prompt.
  #[arg(long)]
  pub c: bool,

  /// Generate a C++ project and skip language prompt.
  #[arg(long)]
  pub cpp: bool,

  /// Generates a C++ project, but uses .cpp2 files instead of .cpp.
  /// This implies --cpp, so it also skips language prompt.
  #[arg(long)]
  pub cpp2: bool,

  #[arg(value_enum, short, long, name = "type")]
  pub subproject_type: Option<CLIProjectOutputTypeIn>
}

/// Generate a new test.
/// Note that all tests are C++ executable subprojects, since only C++
/// test frameworks are currently supported.
#[derive(Args)]
pub struct NewTestProjectCommand {
  /// Project name, no whitespace
  #[arg(required = true)]
  pub new_project_name: String,
}

#[derive(ValueEnum, Clone, Copy)]
pub enum FileCreationLang {
  C,
  Cpp,
  Cpp2
}

#[derive(Args)]
pub struct CreateFilesCommand {
  #[arg(required = true)]
  pub language: FileCreationLang,

  /// Combination of 'h' 's' and 't' (example: hs), where 'h' = Header, 's' = Source,
  /// and 't' = Template implementation
  #[arg(long = "which", default_value = "hs")]
  pub which: String,

  /// Name of the generated file relative to any code folder.
  #[arg(required = true)]
  pub relative_file_names: Vec<String>,

  /// Use '#pragma once' instead of include guards.
  #[arg(short = 'p', long = "use-pragma")]
  pub use_pragma_guards: bool
}

#[derive(Subcommand)]
pub enum DepConfigSubCommand {
  /// Update the dependency configuration repo. Downloads the repo if it is not already present.
  Update(UpdateDependencyConfigsCommand)
}

#[derive(Args)]
pub struct UpdateDependencyConfigsCommand {
  /// Selects the branch to be checked out before pulling changes (or after cloning, if the
  /// repo hasn't been installed yet). If no branch is specified, then the current branch is
  /// updated or the repo is cloned into the 'develop' branch.
  #[arg(long = "to-branch", short = 'b')]
  pub branch: Option<String>,
}

#[derive(ValueEnum, Clone, Copy)]
pub enum UseFileOption {
  #[value(name = "clang-tidy")]
  ClangTidy,

  #[value(name = "clang-format")]
  ClangFormat,

  #[value(name = "gitignore")]
  GitIgnore
}

impl UseFileOption {
  pub fn to_file_name(&self) -> &str {
    match self {
      Self::ClangTidy => ".clang-tidy",
      Self::ClangFormat => ".clang-format",
      Self::GitIgnore => ".gitignore"
    }
  }
}

#[derive(Args)]
pub struct UseFilesCommand {
  /// The file to copy, without the leading '.'
  #[arg(value_enum)]
  pub file: UseFileOption
}

#[derive(Args)]
pub struct TargetInfoCommand {
  /// Select which targets to print info for. Can be in namespace format 'self::the-target'
  /// 'some-project::{ first-target, second-target }', or just a lone target name
  /// 'the-target'. Lone target names only select from targets in the project tree,
  /// but namespaces are able to select dependency targets as well. use the name 'pre-build'
  /// to select the project's pre-build script.
  #[arg(required = true)]
  pub selectors: Vec<String>,

  /// Print the include path of the auto-generated export header
  #[arg(short = 'e')]
  pub export_header: bool,

  /// Prints a target's type (Executable, Static library, etc.).
  #[arg(short = 't')]
  pub item_type: bool
}

#[derive(Args)]
pub struct ProjectInfoCommand {
  /// Select which projects to print info for 
  #[arg(required = false)]
  pub selectors: Vec<String>,

  /// Print the project's full include prefix
  #[arg(short = 'i', long = "include-prefix")]
  pub show_include_prefix: bool,

  /// Recursively list all project targets, including pre-build script and tests
  #[arg(short = 't', long = "list-targets")]
  pub list_targets: bool,

  /// List immediate subprojects
  #[arg(short = 's', long = "subprojects")]
  pub show_subprojects: bool,

  /// Print repository URL
  #[arg(short = 'r', long = "repo-url")]
  pub show_repo_url: bool,

  /// Prints whether the project can be trivially cross compiled
  #[arg(short = 'c', long = "can-cross-compile")]
  pub show_can_trivially_cross_compile: bool,

  /// Prints whether the project supports compilation with Emscripten
  #[arg(long = "supports-emscripten")]
  pub show_supports_emscripten: bool
}

#[derive(Args)]
pub struct PredepInfoCommand {
  /// Select which predefined dependencies to print info for. If no selectors are provided,
  /// then the full list of predefined dependencies is printed out.
  #[arg(required = false)]
  pub selectors: Vec<String>,

  /// List out all the dependency's available targets
  #[arg(short = 't', long = "target-list")]
  pub show_targets: bool,

  /// Print the dependency's git repository URL, if applicable
  #[arg(short = 'r', long = "repo-url")]
  pub show_repository_url: bool,

  /// Print the dependency's GitHub page URL, if applicable
  #[arg(short = 'g', long = "github-url")]
  pub show_github_url: bool,

  /// Prints whether the dependency can be trivially cross-compiled
  #[arg(short = 'c', long = "can-cross-compile")]
  pub show_can_trivially_cross_compile: bool,

  /// Show which download methods the dependency supports, if applicable
  #[arg(short = 'm', long = "download-methods")]
  pub show_supported_download_methods: bool,

  /// Prints a URL to the dependency's documentation README page if it has one
  #[arg(short = 'd', long = "doc-link")]
  pub show_doc_link: bool,

  /// Prints whether the dependency supports can be compiled in a project which supports Emscripten
  #[arg(long = "supports-emscripten")]
  pub show_supports_emscripten: bool,
}

#[derive(Args)]
pub struct ToolInfoCommand {
  /// Print the global GCMake configuration directory
  #[arg(long = "global-config")]
  pub show_config_dir: bool,

  /// Print the dependency cache directory
  #[arg(long = "dep-cache")]
  pub show_dep_cache_dir: bool,

  /// Print the dependency config dir
  #[arg(long = "dep-config")]
  pub show_dep_config_dir: bool
}