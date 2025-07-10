mod cli;
mod commands;
mod gfx;
mod ppm;
mod qoi;
use crate::cli::Cli;
use clap::Parser;

fn main() {
    let command = Cli::parse();
    command.run();
}
