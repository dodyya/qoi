#![allow(unused)]

mod cli;
mod commands;
mod gfx;
mod img;
mod png;
mod ppm;
mod qoi;
mod util;
use crate::cli::Cli;
use clap::Parser;
use std::fs;

fn main() {
    let command = Cli::parse();
    command.run();
}
