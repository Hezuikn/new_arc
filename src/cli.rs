use clap::Parser;

#[derive(Parser)]
#[clap(author, version)]
#[clap(about = "Robocraft CRF archival system")]
pub struct CliArgs {
    /// Display more messages and more details
    #[clap(long)]
    pub verbose: bool,

    /// Path to SQLite database file to use
    #[clap(long)]
    //pub database: Option<String>,
    pub database: String,

    /// Robots per page
    #[clap(short, long)]
    pub size: Option<i64>,

    /// what mode to use
    #[clap(short, long)]
    pub mode: Option<i64>,
}

pub fn parse() -> CliArgs {
    CliArgs::parse()
}
