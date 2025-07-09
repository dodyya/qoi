use pixels::{Pixels, SurfaceTexture};
use std::cmp::min;
use winit::{
    dpi::PhysicalSize,
    event_loop::EventLoop,
    window::{Window, WindowBuilder},
};

pub struct Gfx {
    pub window: Window,
    pixels: Pixels,
    pub width: u32,
    pub height: u32,
}

impl Gfx {
    pub fn new(width: u32, height: u32, title: &str) -> (Self, EventLoop<()>) {
        let pixel_scale = min(1000 / height, 1500 / width);
        let event_loop = EventLoop::new();
        // physical window size = virtual size × scale
        let physical_size = PhysicalSize::new(width * pixel_scale, height * pixel_scale);

        let window = WindowBuilder::new()
            .with_title(title)
            .with_inner_size(physical_size)
            .with_resizable(false)
            .build(&event_loop)
            .unwrap();

        // SurfaceTexture uses the physical (window) pixels,
        // but the 'logical' pixel buffer stays at width×height
        let surface_texture =
            SurfaceTexture::new(physical_size.width, physical_size.height, &window);

        let pixels = Pixels::new(width, height, surface_texture).unwrap();

        (
            Gfx {
                window,
                pixels,
                width,
                height,
            },
            event_loop,
        )
    }

    pub fn render(&mut self) {
        self.pixels.render().unwrap();
    }

    pub fn request_redraw(&mut self) {
        self.window.request_redraw();
    }

    pub fn display(&mut self, bitmap: &[u8]) {
        if bitmap.len() > (self.width * self.height * 4) as usize {
            println!(
                "Had to truncate: {} > {}",
                bitmap.len(),
                self.width * self.height * 4
            );

            self.pixels
                .frame_mut()
                .copy_from_slice(&bitmap[0..(self.width * self.height * 4) as usize]);
        } else if bitmap.len() < (self.width * self.height * 4) as usize {
            println!(
                "Had to pad: {} < {}",
                bitmap.len(),
                self.width * self.height * 4
            );

            let mut padded = vec![0; (self.width * self.height * 4) as usize];
            padded[0..bitmap.len()].copy_from_slice(bitmap);

            self.pixels.frame_mut().copy_from_slice(&padded);
        } else {
            self.pixels
                .frame_mut()
                .copy_from_slice(&bitmap[0..(self.width * self.height * 4) as usize]);
        }
    }
}

fn _rst(frame: &mut [u8]) {
    let black = [0, 0, 0, 255].repeat(frame.len() / 4);
    frame.copy_from_slice(&black)
}
