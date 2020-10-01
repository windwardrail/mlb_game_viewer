mod ui;
mod data;

extern crate sdl2;
extern crate image;

use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use std::time::Duration;
use sdl2::rect::{Rect};
use sdl2::rwops::RWops;
use sdl2::image::{InitFlag, LoadTexture};
use std::path::{PathBuf};

use ui::*;

pub fn main() {
    let mut sdl_renderer = SDL2Renderer::new();
    let mut event_pump = sdl_renderer.context.sdl_context.event_pump().unwrap();

    let mut loading_scene = make_loading_scene(&sdl_renderer);
    loading_scene.accept_visitor(&mut sdl_renderer);
    sdl_renderer.present();

    let mut loaded_scene = make_loaded_scene(&sdl_renderer);

    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit {..} | Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                    break 'running
                },
                _ => loaded_scene.accept_visitor(&mut SDL2EventPropagator::new(event))
            }
        }

        loaded_scene.accept_visitor( &mut sdl_renderer);
        sdl_renderer.present();
        ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
    }
}

fn make_splash(sdl_renderer: &SDL2Renderer) -> Image {
    let splash_path = PathBuf::from("./images/background.png");
    if !splash_path.exists() {
        panic!("{} image could not be found", splash_path.to_str().unwrap());
    }
    let splash = Image::new(splash_path, Position::new(Point::origin(), sdl_renderer.viewport_size()));
    splash
}

fn make_loading_scene(renderer: &SDL2Renderer) -> LayoutItem {
    let splash = make_splash(renderer);

    let loading_message = Text {
        content: "Loading...".to_owned(),
        size: 32,
        pos: Position::new(Point::origin(),Size::new(200, 50)),
        color: Color::new(0, 0, 255)
    };

    let mut c_layout = CenteredLayout::new(Position::new(Point::origin(), renderer.viewport_size()));
    c_layout.add_child(LayoutItem::Widget(WidgetType::Image(splash)));
    c_layout.add_child(LayoutItem::Widget(WidgetType::Text(loading_message)));

    LayoutItem::Layout(Box::new(c_layout))
}

fn make_loaded_scene(renderer: &SDL2Renderer) -> LayoutItem {
    let url = data::make_url_for_date(String::new());
    let games = data::fetch_games(url);

    let splash = make_splash(renderer);

    let mut list_layout = ListLayout::new(Size::new(200, 300));
    for model in games {
        list_layout.add_item(GameItemFactory::make(model));
    }

    let mut v_layout = VCenteredLayout::new(Position::new(Point::origin(), renderer.viewport_size()));

    let mut list_pos = list_layout.position().clone();
    list_pos.upper_left.translate(v_layout.position().center().x - (list_layout.item_size.w / 2) as i32, 0);
    list_layout.set_position(list_pos);

    v_layout.add_child(LayoutItem::Layout(Box::new(list_layout)));

    let mut canvas_layout = CenteredLayout::new(Position::new(Point::origin(), renderer.viewport_size()));
    canvas_layout.add_child(LayoutItem::Widget(WidgetType::Image(splash)));
    canvas_layout.add_child(LayoutItem::Layout(Box::new(v_layout)));

    LayoutItem::Layout(Box::new(canvas_layout))
}

struct SDL2Renderer {
    context: SDLContext,
    canvas: sdl2::render::WindowCanvas,
    coord_reference: Point
}

impl SDL2Renderer {
    fn new() -> Self {
        let context = SDLContext::new();

        let video_subsystem = context.sdl_context.video().unwrap();
        let window = video_subsystem.window("MLB Game Viewer", 960, 540)
            .position_centered()
            .build()
            .unwrap();
        let canvas = window.into_canvas().build().unwrap();

        SDL2Renderer {
            context,
            canvas,
            coord_reference: Point::origin()
        }
    }

    fn viewport_size(&self) -> Size {
        Size::new(self.canvas.viewport().width(), self.canvas.viewport().height())
    }

    fn present(&mut self) {
        self.canvas.present();
    }
}

struct SDLContext {
    sdl_context: sdl2::Sdl,
    font_context: sdl2::ttf::Sdl2TtfContext,
    image_context: sdl2::image::Sdl2ImageContext
}

impl SDLContext {
    fn new() -> Self {
        let sdl_context = sdl2::init().unwrap();
        let font_context = sdl2::ttf::init().unwrap();
        let image_context = sdl2::image::init(InitFlag::PNG | InitFlag::JPG).unwrap();

        SDLContext {
            sdl_context,
            font_context,
            image_context
        }
    }
}

struct SDL2EventPropagator {
    event: sdl2::event::Event
}

impl SDL2EventPropagator {
    pub fn new(event: Event) -> Self {
        SDL2EventPropagator { event }
    }
}

impl Visitor<LayoutItem> for SDL2EventPropagator {
    fn visit_element(&mut self, element: &mut LayoutItem) {
        match element {
            LayoutItem::Layout(layout) => { self.visit_element(layout) }
            LayoutItem::Widget(widget) => { self.visit_element(widget) }
        }
    }
}

impl Visitor<Box<dyn Layout>> for SDL2EventPropagator {
    fn visit_element(&mut self, element: &mut Box<dyn Layout>) {
        let consumed = match self.event {
            Event::KeyDown { keycode: Some(code), .. } => {
                if let Some(key) = match code {
                    Keycode::Right => Some(Key::Right),
                    Keycode::Left => Some(Key::Left),
                    _ => { None }
                } {
                    element.handle_key(key)
                } else { false }
            },
            _ => { false }
        };

        if ! consumed {
            let mut i = 0;
            while let Some(child) = element.child_at(i) {
                self.visit_element(child);
                i += 1;
            }
        }
    }
}

impl Visitor<WidgetType> for SDL2EventPropagator {
    fn visit_element(&mut self, _: &mut WidgetType) { }
}

impl Visitor<Frame> for SDL2Renderer {
    fn visit_element(&mut self, element: &mut Frame) {
        if let Some(color) = &element.color() {
            self.canvas.set_draw_color(sdl2::pixels::Color::RGB(color.r as u8, color.g as u8, color.b as u8));
        }
        let Point { x, y } = translate_to_global(&element.position().upper_left, &self.coord_reference);
        // println!("Drawing frame at {}, {}", x, y);
        self.canvas.fill_rect(sdl2::rect::Rect::new(x as i32, y as i32, element.position().size.w, element.position().size.h)).unwrap();
    }
}

impl Visitor<Image> for SDL2Renderer {
    fn visit_element(&mut self, element: &mut Image) {
        let creator = self.canvas.texture_creator();

        let texture = match &element.source {
            ImageSource::Path(path) => {
                match creator.load_texture(path) {
                    Ok(texture) => { Some(texture) }
                    Err(_) => { None }
                }
            }
            ImageSource::Bytes(bytes) => {
                match creator.load_texture_bytes(bytes.as_slice()) {
                    Ok(texture) => { Some(texture) }
                    Err(_) => { None }
                }
            }
        };
        let mut dest_pos = element.position().clone();
        dest_pos.upper_left = translate_to_global(&dest_pos.upper_left, &self.coord_reference);
        let Position {upper_left: Point {x, y}, size: Size { w, h }} = dest_pos;
        match texture {
            None => {
                self.canvas.set_draw_color(sdl2::pixels::Color::GRAY);
                self.canvas.fill_rect(Rect::new(x as i32, y as i32, w, h)).unwrap();
            }
            Some(t) => {
                self.canvas.copy(&t, None, Rect::new(x as i32, y as i32, w, h)).unwrap();
            }
        }
    }
}

impl Visitor<Text> for SDL2Renderer {
    fn visit_element(&mut self, element: &mut Text) {
        let Color { r, g, b} = element.color;
        let font_bytes = include_bytes!("../fonts/LeagueGothic-Regular.otf");
        let font = self.context.font_context.load_font_from_rwops(RWops::from_bytes(font_bytes).unwrap(), element.size as u16).unwrap();
        if let Ok(font_surface) = font.render(element.content.as_str()).blended_wrapped(sdl2::pixels::Color::RGB(r as u8, g as u8, b as u8), element.position().size.w) {
            let (rendered_w, rendered_h) = font_surface.size();
            let translated_center = translate_to_global(&element.position().center(), &self.coord_reference);
            let dest_rect = Rect::new(translated_center.x - (rendered_w / 2) as i32, translated_center.y - (rendered_h / 2) as i32, rendered_w, rendered_h);
            self.canvas.copy(&font_surface.as_texture(&self.canvas.texture_creator()).unwrap(), None, dest_rect).unwrap();
        }
    }
}

impl Visitor<LayoutItem> for SDL2Renderer {
    fn visit_element(&mut self, element: &mut LayoutItem) {
        match element {
            LayoutItem::Layout(l) => { self.visit_element(l) }
            LayoutItem::Widget(w) => { self.visit_element(w) }
        }
    }
}

impl Visitor<Box<dyn Layout>> for SDL2Renderer {
    fn visit_element(&mut self, element: &mut Box<dyn Layout>) {
        let mut i = 0;
        self.coord_reference = translate_to_global(&element.position().upper_left, &self.coord_reference);
        // println!("referent coord moved to {:?}", self.coord_reference);
        while let Some(child) = element.child_at(i) {
            self.visit_element(child);
            i += 1;
        }
        self.coord_reference = translate_to_relative(&self.coord_reference, &element.position().upper_left);
    }
}

impl Visitor<WidgetType> for SDL2Renderer {
    fn visit_element(&mut self, element: &mut WidgetType) {
        match element {
            WidgetType::Frame(frame) => { self.visit_element(frame) }
            WidgetType::Image(image) => { self.visit_element(image) }
            WidgetType::Text(text) => { self.visit_element(text) }
        }
    }
}


