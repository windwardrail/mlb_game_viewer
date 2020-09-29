extern crate sdl2;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use std::time::Duration;
use sdl2::rect::{Rect};
use sdl2::ttf::Sdl2TtfContext;
use sdl2::rwops::RWops;
use sdl2::gfx::primitives::DrawRenderer;
use sdl2::surface::Surface;
use sdl2::image::{InitFlag, ImageRWops, LoadTexture, LoadSurface};
use std::path::{Path, PathBuf};
use sdl2::video::Window;
use sdl2::render::Canvas;

pub fn main() {
    let mut sdl_renderer = SDL2Renderer::new();
    let mut event_pump = sdl_renderer.context.sdl_context.event_pump().unwrap();

    let splash_path = PathBuf::from("./images/background.png");
    if ! splash_path.exists() {
        panic!("{} image could not be found", splash_path.to_str().unwrap());
    }
    let splash = Image {
        source: splash_path,
        pos: Position { upper_left: Point::origin(), size: Size {w: sdl_renderer.canvas.viewport().width(), h: sdl_renderer.canvas.viewport().height()} }
    };
    // println!("initial image position: {:?}", splash.pos);

    let message = Text {
        content: "Hello Disney Plus".to_owned(),
        size: 32,
        pos: Position {
            upper_left: Point::origin(),
            size: Size { w: 200, h: 50 }
        },
        color: Color {
            r: 0,
            g: 255,
            b: 0
        }
    };

    let mut list_layout = ListLayout::new(Size{ w: 300, h: 200 });
    list_layout.add_item(ListItemFactory::make());
    list_layout.add_item(ListItemFactory::make());
    list_layout.add_item(ListItemFactory::make());

    let mut v_layout = VCenteredLayout::new(Position {
        upper_left: Point::origin(),
        size: sdl_renderer.viewport_size()
    });
    list_layout.position.upper_left.translate(v_layout.position.center().x - (list_layout.item_size.w / 2) as i32, 0);
    v_layout.add_child(LayoutItem::Layout(Box::new(list_layout)));

    let mut canvas_layout = CenteredLayout::new(Position {
        upper_left: Point::origin(),
        size: sdl_renderer.viewport_size()
    });
    canvas_layout.add_child(LayoutItem::Widget(WidgetType::Image(splash)));
    canvas_layout.add_child(LayoutItem::Layout(Box::new(v_layout)));
    let root_item = LayoutItem::Layout(Box::new(canvas_layout));

    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit {..} | Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                    break 'running
                },
                Event::KeyDown { keycode: Some(Keycode::Right), .. } => {},
                _ => {}
            }
        }

        root_item.accept_visitor( &mut sdl_renderer);
        sdl_renderer.present();
        ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
    }
}

struct ListItemFactory {}

impl ListItemFactory {
    fn make() -> ListItem {
        let mut item = Frame::new();
        item.fill = Some(Color::new(255, 0, 0 ));
        let mut selected = Frame::new();
        selected.fill = Some(Color::new(0, 255, 0));
        ListItem::new(LayoutItem::Widget(WidgetType::Frame(item)), LayoutItem::Widget(WidgetType::Frame(selected)))
    }
}

#[derive(Debug)]
struct Size {
    w: u32,
    h: u32
}

impl Clone for Size {
    fn clone(&self) -> Self {
        let Size { w, h } = *self;
        Size { w, h }
    }
}

#[derive(Debug)]
struct Point {
    x: i32,
    y: i32
}

impl Point {
    pub fn new(x: i32, y: i32) -> Self {
        Point { x, y }
    }

    pub fn origin() -> Self {
        Point { x: 0, y: 0 }
    }

    pub fn translate(&mut self, x: i32, y: i32) {
        self.x += x;
        self.y += y;
    }
}

#[derive(Debug)]
struct Position {
    upper_left: Point,
    size: Size
}

impl Position {
    fn new() -> Self {
        Position { upper_left: Point { x: 0, y: 0 }, size: Size { w: 0, h: 0 } }
    }

    fn center(&self) -> Point {
        Point {
            x: self.upper_left.x + (self.size.w / 2) as i32,
            y: self.upper_left.y + (self.size.h / 2) as i32
        }
    }
}

impl Clone for Position {
    fn clone(&self) -> Self {
        let Position{ upper_left: Point { x, y }, size: Size {  w, h } } = *self;
        Position {
            upper_left: Point { x, y },
            size: Size { w, h }
        }
    }
}

struct Color {
    r: u32,
    g: u32,
    b: u32
}

impl Color {
    pub fn new(r: u32, g: u32, b: u32) -> Self {
        Color { r, g, b }
    }
}

#[derive(Debug)]
struct Image {
    source: PathBuf,
    pos: Position
}

struct Text {
    content: String,
    size: u32,
    pos: Position,
    color: Color
}

struct Frame {
    pos: Position,
    fill: Option<Color>,
    border: Option<Color>,
    border_width: u32,
    radius: u32
}

impl Frame {
    pub fn new() -> Self {
        Frame {
            pos: Position::new(),
            fill: None,
            border: None,
            border_width: 0,
            radius: 0
        }
    }
}

struct CenteredLayout {
    children: Vec<LayoutItem>,
    position: Position,
    center: Point
}

impl CenteredLayout {
    fn new(position: Position) -> Self {
        CenteredLayout {
            children: vec![],
            center: Point{x: position.upper_left.x + (position.size.w / 2) as i32, y: position.upper_left.y + (position.size.h / 2) as i32},
            position
        }
    }

    fn add_child(&mut self, mut item: LayoutItem) {
        let new_pos = Position {
            upper_left: Point { x: self.center.x - (item.position().size.w / 2) as i32, y: self.center.y - (item.position().size.h / 2) as i32 },
            size: item.position().size.clone()
        };
        item.set_position(new_pos);
        println!("centered layout child pos: {:?}", item.position());
        self.children.push(item);
    }
}

struct VCenteredLayout {
    children: Vec<LayoutItem>,
    position: Position,
}

impl VCenteredLayout {
    fn new(position: Position) -> Self {
        VCenteredLayout {
            children: vec![],
            position
        }
    }

    fn add_child(&mut self, mut item: LayoutItem) {
        let item_pos = item.position();
        let new_pos = Position {
            upper_left: Point { x: item_pos.upper_left.x, y: self.position.center().y - (item_pos.size.h / 2) as i32},
            size: item_pos.size.clone()
        };
        item.set_position(new_pos);
        self.children.push(item);
    }
}

struct ListLayout {
    children: Vec<ListItem>,
    position: Position,
    item_size: Size,
    spacing: u32,
    selected: usize
}

impl ListLayout {
    pub fn new(item_size: Size) -> Self {
        ListLayout {
            children: vec![],
            position: Position {
                upper_left: Point::origin(),
                size: Size { w: 0, h: item_size.h }
            },
            item_size,
            spacing: 20,
            selected: 0
        }
    }

    fn add_item(&mut self, mut item: ListItem) {
        item.set_position(Position {
            upper_left: Point {
                x: self.position.size.w as i32 + self.spacing as i32,
                y: 0
            },
            size: self.item_size.clone()
        });
        println!("item pos: {:?}", item.position());
        self.position.size.w += (self.spacing + self.item_size.w);
        self.children.push(item);
    }
}

struct ListItem {
    item: LayoutItem,
    selected_item: LayoutItem,
    pos: Position
}

impl ListItem {
    fn new(unselected: LayoutItem, selected: LayoutItem) -> Self {
        ListItem {
            item: unselected,
            selected_item: selected,
            pos: Position::new()
        }
    }
}

enum LayoutItem {
    Layout(Box<dyn Layout>),
    Widget(WidgetType)
}

enum WidgetType {
    Frame(Frame),
    Image(Image),
    Text(Text)
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
        Size {
            w: self.canvas.viewport().width(),
            h: self.canvas.viewport().height()
        }
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
        let image_context = sdl2::image::init(sdl2::image::InitFlag::all()).unwrap();

        SDLContext {
            sdl_context,
            font_context,
            image_context
        }
    }
}

trait Widget: Positionable {
    fn get_type(&self) -> WidgetType;
}

trait Layout: Positionable {
    fn child_at(&self, index: usize) -> Option<&LayoutItem>;
}

impl Layout for ListItem {
    fn child_at(&self, _: usize) -> Option<&LayoutItem> {
        unimplemented!()
    }
}

impl Layout for CenteredLayout {
    fn child_at(&self, index: usize) -> Option<&LayoutItem> {
        self.children.get(index)
    }
}

impl Layout for VCenteredLayout {
    fn child_at(&self, index: usize) -> Option<&LayoutItem> {
        self.children.get(index)
    }
}

impl Layout for ListLayout {
    fn child_at(&self, index: usize) -> Option<&LayoutItem> {
        match self.children.get(index) {
            None => None,
            Some(item) => match index == self.selected {
                true => Some(&item.selected_item),
                false => Some(&item.item)
            }
        }
    }
}

trait Positionable {
    fn set_position(&mut self, pos: Position);
    fn position(&self) -> &Position;
}

impl Positionable for CenteredLayout {
    fn set_position(&mut self, pos: Position) {
        self.position = pos;
    }

    fn position(&self) -> &Position {
        &self.position
    }
}

impl Positionable for VCenteredLayout {
    fn set_position(&mut self, pos: Position) {
        self.position = pos;
    }

    fn position(&self) -> &Position {
        &self.position
    }
}

impl Positionable for ListLayout {
    fn set_position(&mut self, pos: Position) {
        self.position = pos;
    }

    fn position(&self) -> &Position {
        &self.position
    }
}

impl Positionable for ListItem {
    fn set_position(&mut self, pos: Position) {
        self.item.set_position(pos.clone());
        self.selected_item.set_position(pos);
    }

    fn position(&self) -> &Position {
        self.item.position()
    }
}

impl Positionable for LayoutItem {
    fn set_position(&mut self, pos: Position) {
        match self {
            LayoutItem::Layout(l) => { l.set_position(pos) }
            LayoutItem::Widget(w) => { w.set_position(pos) }
        }
    }

    fn position(&self) -> &Position {
        match self {
            LayoutItem::Layout(l) => { l.position() }
            LayoutItem::Widget(w) => { w.position() }
        }
    }
}

impl Positionable for WidgetType {
    fn set_position(&mut self, pos: Position) {
        match self {
            WidgetType::Frame(f) => { f.pos = pos }
            WidgetType::Image(i) => { i.pos = pos }
            WidgetType::Text(t) => { t.pos = pos }
        }
    }

    fn position(&self) -> &Position {
        match self {
            WidgetType::Frame(f) => { &f.pos }
            WidgetType::Image(i) => { &i.pos }
            WidgetType::Text(t) => { &t.pos }
        }
    }
}

trait Visitee<T> {
    fn accept_visitor(&self, visitor: &mut dyn Visitor<T>);
}

impl<T> Visitee<T> for T {
    fn accept_visitor(&self, visitor: &mut dyn Visitor<T>) {
        visitor.visit_element(self);
    }
}

trait Visitor<T> {
    fn visit_element(&mut self, element: &T);
}

impl Visitor<Frame> for SDL2Renderer {
    fn visit_element(&mut self, element: &Frame) {
        if let Some(color) = &element.fill {
            self.canvas.set_draw_color(sdl2::pixels::Color::RGB(color.r as u8, color.g as u8, color.b as u8));
        }
        let Point { x, y } = translate_to_global(&element.pos.upper_left, &self.coord_reference);
        // println!("Drawing frame at {}, {}", x, y);
        self.canvas.fill_rect(sdl2::rect::Rect::new(x as i32, y as i32, element.pos.size.w, element.pos.size.h));
    }
}

impl Visitor<Image> for SDL2Renderer {
    fn visit_element(&mut self, element: &Image) {
        let creator = self.canvas.texture_creator();
        let bg_texture = creator.load_texture(&element.source).unwrap();
        let mut dest_pos = element.pos.clone();
        dest_pos.upper_left = translate_to_global(&dest_pos.upper_left, &self.coord_reference);
        let Position {upper_left: Point {x, y}, size: Size { w, h }} = dest_pos;
        self.canvas.copy(&bg_texture, None, Rect::new(x as i32, y as i32, w, h));
    }
}

impl Visitor<Text> for SDL2Renderer {
    fn visit_element(&mut self, element: &Text) {
        let Color { r, g, b} = element.color;
        let Position { upper_left: Point { x, y}, size: Size { w, h} } = element.pos;
        let font_bytes = include_bytes!("../fonts/LeagueGothic-Regular.otf");
        let font = self.context.font_context.load_font_from_rwops(RWops::from_bytes(font_bytes).unwrap(), 48).unwrap();
        let font_surface = font.render(element.content.as_str()).solid(sdl2::pixels::Color::RGB(r as u8, g as u8, b as u8)).unwrap();
        let src_rect = Rect::new(0, 0, w, h);
        let dest_rect = Rect::new(x as i32, y as i32, w, h);
        self.canvas.copy(&font_surface.as_texture(&self.canvas.texture_creator()).unwrap(), None, dest_rect);
    }
}

impl Visitor<LayoutItem> for SDL2Renderer {
    fn visit_element(&mut self, element: &LayoutItem) {
        match element {
            LayoutItem::Layout(l) => { self.visit_element(l) }
            LayoutItem::Widget(w) => { self.visit_element(w) }
        }
    }
}

impl Visitor<Box<dyn Layout>> for SDL2Renderer {
    fn visit_element(&mut self, element: &Box<dyn Layout>) {
        let mut i = 0;
        self.coord_reference = translate_to_global(&element.position().upper_left, &self.coord_reference);
        // println!("referent coord moved to {:?}", self.coord_reference);
        while let Some(child) = element.child_at(i) {
            self.visit_element(child);
            i += 1;
        }
        self.coord_reference = translate_to_relative(&self.coord_reference, &element.position().upper_left);
        // println!("referent coord moved back to {:?}", self.coord_reference);
    }
}

impl Visitor<WidgetType> for SDL2Renderer {
    fn visit_element(&mut self, element: &WidgetType) {
        match element {
            WidgetType::Frame(frame) => { self.visit_element(frame) }
            WidgetType::Image(image) => { self.visit_element(image) }
            WidgetType::Text(text) => { self.visit_element(text) }
        }
    }
}

fn translate_to_global(relative: &Point, global: &Point) -> Point {
    Point {
        x: global.x + relative.x,
        y: global.y + relative.y
    }
}

fn translate_to_relative(global: &Point, relative: &Point) -> Point {
    Point {
        x: global.x - relative.x,
        y: global.y - relative.y
    }
}
