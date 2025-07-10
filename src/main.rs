#![allow(unused)]

mod cli;
mod commands;
mod gfx;
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
    // let file_path = "pics/dice.png";
    // let img_result = fs::read(&file_path);

    // let (width, height, pixel_buf) = png::parse_img(img_result.unwrap().into_iter());
}
