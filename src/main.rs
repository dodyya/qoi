mod gfx;
mod ppm;
mod qoi;
use std::env;
use std::fs;
use std::slice::Chunks;
use winit::{
    event::{Event, WindowEvent},
    event_loop::ControlFlow,
};

const PATH: &str = "./pics/testcard.ppm";

fn main() {
    let img = load_img(PATH).into_iter();
    let (width, height, pixel_buf): (u32, u32, Vec<u8>) = ppm::parse_img(img);

    let img_qoi = qoi::encode_img(width, height, pixel_buf);

    fs::write("./output/testcard.qoi", img_qoi);

    display_image();
}

fn load_img(path: &str) -> Vec<u8> {
    fs::read(path).unwrap()
}

fn display_image() {
    let path = env::args()
        .nth(1)
        .unwrap_or("/Users/dodya/git/qoi/pics/wikipedia_008.qoi".into());
    let img = load_img(&path).into_iter();
    let (width, height, pixel_buf): (u32, u32, Vec<u8>);
    if path.ends_with("qoi") {
        (width, height, pixel_buf) = qoi::parse_img(img);
    } else {
        (width, height, pixel_buf) = ppm::parse_img(img);
    }

    let (mut gfx, event_loop) = gfx::Gfx::new(width, height, &path);
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
