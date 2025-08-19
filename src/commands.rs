use crate::gfx;
use crate::img::RawImage;
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
    Open { file_path: PathBuf },
    /// Convert between image formats (.qoi, .ppm, .png)
    Convert {
        files: Vec<PathBuf>,
        #[arg(short, long, help = "Output file path (for single file conversion)")]
        output: Option<PathBuf>,
        #[arg(
            short = 't',
            long = "target",
            help = "Target file extension for batch conversion (qoi, ppm, png)"
        )]
        target_extension: Option<String>,
    },
    /// Create a .qoi or .ppm image from a dimension-prefixed RGBA byte stream stdin
    Write {
        output_path: PathBuf,
        #[arg(short, long, default_value_t = false)]
        forever: bool,
        #[arg(short, long, default_value_t = true)]
        numbered: bool,
    },
    /// View a dimension-prefixed RGBA byte stream in stdin
    View,
}

impl Command {
    pub fn run(self) -> Result<(), String> {
        match self {
            Command::Open { file_path } => open(&file_path),
            Command::Convert {
                files,
                output,
                target_extension,
            } => convert(&files, output.as_ref(), target_extension.as_ref()),
            Command::Write {
                output_path,
                forever,
                numbered,
            } => write(forever, numbered, &output_path),
            Command::View => view(),
        }
    }
}

fn open(file_path: &PathBuf) -> Result<(), String> {
    let img_result = fs::read(&file_path);
    if let Err(e) = img_result {
        return Err(e.to_string());
    }

    let img: RawImage;
    if file_path.extension().unwrap_or_default() == "qoi" {
        img = qoi::parse_img(img_result.unwrap().into_iter());
    } else if file_path.extension().unwrap_or_default() == "ppm" {
        img = ppm::parse_img(img_result.unwrap().into_iter());
    } else if file_path.extension().unwrap_or_default() == "png" {
        img = png::parse_img(img_result.unwrap().into_iter());
    } else {
        return Err(
            "Invalid file extension provided. Only .ppm, .qoi, and .png are supported".into(),
        );
    }

    display(img, file_path.to_str().unwrap());
    Ok(())
}

fn display(img: RawImage, title: &str) {
    let RawImage(width, height, pixel_buf) = img;
    let (mut gfx, event_loop) = gfx::Gfx::new(width, height, title);
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

fn convert(
    files: &[PathBuf],
    output: Option<&PathBuf>,
    target_extension: Option<&String>,
) -> Result<(), String> {
    if files.len() < 1 {
        return Err("At least one input file is required".into());
    }

    if files.len() == 1 && output.is_some() {
        return convert_single(&files[0], output.unwrap());
    }

    if files.len() == 2 && output.is_some() {
        return convert_single(&files[0], output.unwrap());
    }

    if files.len() >= 3 {
        let first_ext = files[0].extension().unwrap_or_default();
        for file in files.iter() {
            let ext = file.extension().unwrap_or_default();
            assert_eq!(
                ext, first_ext,
                "All input files must have the same extension"
            );
        }

        let target_ext = if let Some(target) = target_extension {
            target.as_str()
        } else {
            match first_ext.to_str().unwrap_or("") {
                "ppm" => "qoi",
                "qoi" => "ppm",
                "png" => "qoi",
                _ => "ppm",
            }
        };

        for file_path in files {
            let output_path = file_path.with_extension(target_ext);
            convert_single(file_path, &output_path)?;
        }
        return Ok(());
    }

    Err("Invalid arguments: provide either 1-2 files with --output, or 3+ files with same extension".into())
}

fn convert_single(file_path: &PathBuf, output_path: &PathBuf) -> Result<(), String> {
    let img_result = fs::read(&file_path);
    if let Err(e) = img_result {
        return Err(e.to_string());
    }

    let input_ext = file_path
        .extension()
        .unwrap_or_default()
        .to_str()
        .unwrap_or("");
    let output_ext = output_path
        .extension()
        .unwrap_or_default()
        .to_str()
        .unwrap_or("");

    let img = match input_ext {
        "ppm" => ppm::parse_img(img_result.unwrap().into_iter()),
        "qoi" => qoi::parse_img(img_result.unwrap().into_iter()),
        "png" => png::parse_img(img_result.unwrap().into_iter()),
        _ => return Err("Unsupported input format".into()),
    };

    let encoded_data = match output_ext {
        "ppm" => ppm::encode_img(img),
        "qoi" => qoi::encode_img(img),
        "png" => png::encode_img(img),
        _ => return Err("Unsupported output format".into()),
    };

    fs::write(output_path, encoded_data).map_err(|e| e.to_string())
}

fn write(forever: bool, numbered: bool, output_path: &PathBuf) -> Result<(), String> {
    use std::io::{self, Read};

    let mut input = io::BufReader::new(io::stdin());
    let extension = output_path
        .extension()
        .unwrap_or_default()
        .to_str()
        .unwrap();

    let path = output_path.parent().ok_or("No parent directory")?;
    let stem = output_path
        .file_stem()
        .unwrap_or_default()
        .to_str()
        .unwrap();

    let mut n = 0;
    loop {
        n += 1;
        let mut w_buf = [0u8; 4];
        let mut h_buf = [0u8; 4];
        input.read_exact(&mut w_buf);
        input.read_exact(&mut h_buf);
        let w = u32::from_be_bytes(w_buf);
        let h = u32::from_be_bytes(h_buf);
        let image_size = (w as usize)
            .checked_mul(h as usize)
            .and_then(|s| s.checked_mul(4))
            .ok_or("Image dimensions too large")?;
        let mut image_data = vec![0u8; image_size];
        if let Err(e) = input.read_exact(&mut image_data) {
            return Err(e.to_string());
        }

        let img = RawImage(w, h, image_data);

        let out_path = if numbered {
            PathBuf::from(format!(
                "{}/{}{:0>5}.{}",
                path.display(),
                stem,
                n,
                extension
            ))
        } else {
            PathBuf::from(format!("{}/{}.{}", path.display(), stem, extension))
        };

        let result = match extension {
            "qoi" => fs::write(out_path, qoi::encode_img(img)).map_err(|e| e.to_string()),
            "ppm" => fs::write(out_path, ppm::encode_img(img)).map_err(|e| e.to_string()),
            "png" => fs::write(out_path, png::encode_img(img)).map_err(|e| e.to_string()),
            _ => Err("Unsupported output format.".into()),
        };

        if let Err(e) = result {
            return Err(e);
        }

        if !forever {
            return Ok(());
        }
    }
}

fn view() -> Result<(), String> {
    use std::io::{self, Read};

    let mut input = io::BufReader::new(io::stdin());

    let mut w_buf = [0u8; 4];
    let mut h_buf = [0u8; 4];
    input.read_exact(&mut w_buf);
    input.read_exact(&mut h_buf);
    let w = u32::from_be_bytes(w_buf);
    let h = u32::from_be_bytes(h_buf);
    let mut image_data = vec![0u8; (w * h * 4) as usize];
    if let Err(e) = input.read_exact(&mut image_data) {
        return Err(e.to_string());
    }

    let img = RawImage(w, h, image_data);

    display(img, "Piped image");
    Ok(())
}
