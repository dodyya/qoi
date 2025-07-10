use crate::commands::Command;
use clap::Parser;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    command: Command,
}

impl Cli {
    pub fn run(self) {
        let result = self.command.run();
        match result {
            Ok(_) => {}
            Err(e) => println!("Error: {}", e),
        }
    }
}
