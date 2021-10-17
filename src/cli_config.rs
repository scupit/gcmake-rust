use clap::{AppSettings, Clap};

const SKY: &'static str = "Skylar Cupit";

#[derive(Clap)]
#[clap(version = "1.0", author = SKY)]
#[clap(setting = AppSettings::ColoredHelp)]
pub struct Opts {
  #[clap(default_value = ".")]
  pub project_root: String,

  #[clap(subcommand)]
  pub subcommand: Option<SubCommand>
}

#[derive(Clap)]
pub enum SubCommand {
    #[clap(version = "1.0", author = SKY)]
    New(NewProjectCommand)
}

/// Generate a new project
#[derive(Clap)]
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
