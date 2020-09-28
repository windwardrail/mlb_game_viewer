extern crate sdl2;

use sdl2::pixels::Color;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use std::time::Duration;
use sdl2::rect::{Rect, Point};
use sdl2::ttf::Sdl2TtfContext;
use sdl2::rwops::RWops;
use sdl2::gfx::primitives::DrawRenderer;
use sdl2::surface::Surface;
use sdl2::image::{InitFlag, ImageRWops, LoadTexture, LoadSurface};
use std::path::Path;

pub fn main() {
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
 
    let window = video_subsystem.window("rust-sdl2 demo", 800, 600)
        .position_centered()
        .build()
        .unwrap();
 
    let mut canvas = window.into_canvas().build().unwrap();
    let texture_creator = canvas.texture_creator();

    let ttf_context = sdl2::ttf::init().unwrap();
    let font_bytes = include_bytes!("../fonts/LeagueGothic-Regular.otf");
    let font = ttf_context.load_font_from_rwops(RWops::from_bytes(font_bytes).unwrap(), 48).unwrap();
    let font_surface = font.render("Hello Disney+").solid(Color::WHITE).unwrap();

    let image_context = sdl2::image::init(sdl2::image::InitFlag::all()).unwrap();
    let bg_path = Path::new("./images/background.png");
    if ! bg_path.exists() {
        panic!("background image could not be found");
    }
    let bg_texture = texture_creator.load_texture(bg_path).unwrap();
    let bg_surface = sdl2::surface::Surface::from_file(bg_path).unwrap();


    let mut event_pump = sdl_context.event_pump().unwrap();
    let mut i = 0;
    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit {..} |
                Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                    break 'running
                },
                _ => {}
            }
        }

        canvas.set_draw_color(Color::RGB(0, 0, 0));
        canvas.clear();

        canvas.copy(&bg_texture, None, None);

        canvas.set_draw_color(Color::RGB(64, 128, 255));
        canvas.fill_rect(Rect::from_center(canvas.viewport().center(), 300, 200));

        canvas.copy(&font_surface.as_texture(&canvas.texture_creator()).unwrap(), font_surface.rect(), font_surface.rect());

        canvas.present();
        ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
    }
}


