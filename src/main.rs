use clap::{Parser, Subcommand};
use parse_obsidian::tasks::{find_tasks, parse_tasks};
use std::io::{self, Read};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "parse-obsidian")]
#[command(about = "Parse Obsidian vault files", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Extract tasks from files or text
    Tasks {
        /// File or directory path to parse. If omitted, reads from stdin.
        path: Option<PathBuf>,
    },
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Tasks { path } => {
            let tasks = match path {
                Some(p) => find_tasks(&p)?,
                None => {
                    let mut input = String::new();
                    io::stdin().read_to_string(&mut input)?;
                    parse_tasks(&input)
                }
            };
            println!("{}", serde_json::to_string_pretty(&tasks)?);
        }
    }

    Ok(())
}
