use crate::gfx;
use crate::png;
use crate::ppm;
use crate::qoi;
use clap::Subcommand;
use std::env;
use std::fs;
use std::fs::File;
use std::io::BufReader;
use std::io::BufWriter;
use std::io::prelude::*;
use std::path::PathBuf;
use winit::{
    event::{Event, WindowEvent},
    event_loop::ControlFlow,
};
#[derive(Debug, Subcommand)]
pub enum Command {
    /// Display a .ppm or .qoi image
    Display { file_path: PathBuf },
    /// Convert a .qoi to a .ppm or vice versa
    Convert {
        file_path: PathBuf,
        ///The target location
        output_path: PathBuf,
    },
}

impl Command {
    pub fn run(self) -> Result<(), String> {
        match self {
            Command::Display { file_path } => display(file_path),
            Command::Convert {
                file_path,
                output_path,
            } => convert(file_path, output_path),
        }
    }
}

fn display(file_path: PathBuf) -> Result<(), String> {
    let img_result = fs::read(&file_path);
    if let Err(e) = img_result {
        return Err(e.to_string());
    }

    let (width, height, pixel_buf): (u32, u32, Vec<u8>);
    if file_path.extension().unwrap_or_default() == "qoi" {
        (width, height, pixel_buf) = qoi::parse_img(img_result.unwrap().into_iter());
    } else if file_path.extension().unwrap_or_default() == "ppm" {
        (width, height, pixel_buf) = ppm::parse_img(img_result.unwrap().into_iter());
    } else if file_path.extension().unwrap_or_default() == "png" {
        (width, height, pixel_buf) = png::parse_img(img_result.unwrap().into_iter());
    } else {
        return Err("Invalid file extension provided. Only .ppm and .qoi are supported".into());
    }

    let (mut gfx, event_loop) = gfx::Gfx::new(width, height, file_path.to_str().unwrap());
    gfx.display(&pixel_buf);
    gfx.render();
    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;

        if let Event::WindowEvent {
            event: WindowEvent::CloseRequested,
            ..
        } = event
        {
            *control_flow = ControlFlow::Exit;
        }
    });
}

fn convert(file_path: PathBuf, output_path: PathBuf) -> Result<(), String> {
    let img_result = fs::read(&file_path);
    if let Err(e) = img_result {
        return Err(e.to_string());
    }

    if file_path.extension().unwrap_or_default() == "ppm"
        && output_path.extension().unwrap_or_default() == "qoi"
    {
        let (width, height, pixels) = ppm::parse_img(img_result.unwrap().into_iter());
        let write_result = fs::write(output_path, qoi::encode_img(width, height, pixels));
        if let Err(e) = write_result {
            return Err(e.to_string());
        } else {
            return Ok(());
        }
    } else if file_path.extension().unwrap_or_default() == "qoi"
        && output_path.extension().unwrap_or_default() == "ppm"
    {
        let (width, height, pixels) = qoi::parse_img(img_result.unwrap().into_iter());
        let write_result = fs::write(output_path, ppm::encode_img(width, height, pixels));
        if let Err(e) = write_result {
            return Err(e.to_string());
        } else {
            return Ok(());
        }
    } else {
        return Err("Something went wrong with your file extensions.".into());
    }
}
