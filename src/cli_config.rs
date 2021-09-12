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
    New(CommandNew)
}

/// Noice other one
#[derive(Clap)]
pub struct CommandNew {
    /// Do something with debug
    #[clap(required = true)]
    pub new_project_root: String
}
