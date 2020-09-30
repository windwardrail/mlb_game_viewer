pub mod layouts;

use std::path::{PathBuf};
use json::JsonValue;

#[derive(Debug)]
pub enum Key {
    Right,
    Left
}

#[derive(Debug)]
pub(crate) struct Size {
    pub(crate) w: u32,
    pub h: u32
}

impl Size {
    pub fn new(w: u32, h: u32) -> Self {
        Size { w, h }
    }
}

impl Clone for Size {
    fn clone(&self) -> Self {
        let Size { w, h } = *self;
        Size { w, h }
    }
}

#[derive(Debug)]
pub(crate) struct Point {
    pub x: i32,
    pub y: i32
}

impl Point {
    pub(crate) fn new(x: i32, y: i32) -> Self {
        Point { x, y }
    }

    pub(crate) fn origin() -> Self {
        Point { x: 0, y: 0 }
    }

    pub(crate) fn translate(&mut self, x: i32, y: i32) {
        self.x += x;
        self.y += y;
    }
}

pub struct Color {
    pub(crate) r: u32,
    pub(crate) g: u32,
    pub(crate) b: u32
}

impl Color {
    pub fn new(r: u32, g: u32, b: u32) -> Self {
        Color { r, g, b }
    }
}

#[derive(Debug)]
pub struct Position {
    pub(crate) upper_left: Point,
    pub(crate) size: Size
}

impl Position {
    pub(crate) fn new(upper_left: Point, size: Size) -> Self {
        Position { upper_left, size }
    }

    pub(crate) fn center(&self) -> Point {
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

pub struct Image {
    pub(crate) source: ImageSource,
    pos: Position,
    preserve_aspect_w: bool
}

pub enum ImageSource {
    Path(PathBuf),
    Bytes(Vec<u8>)
}

impl Image {
    pub fn new(source: PathBuf, pos: Position) -> Self {
        Image {
            source: ImageSource::Path(source),
            pos,
            preserve_aspect_w: true
        }
    }

    pub fn from_bytes(data: Vec<u8>, pos: Position) -> Self {
        Image {
            source: ImageSource::Bytes(data),
            pos,
            preserve_aspect_w: true
        }
    }
}

pub struct Text {
    pub content: String,
    pub size: u32,
    pub pos: Position,
    pub color: Color
}

pub struct Frame {
    pos: Position,
    fill: Option<Color>,
    border: Option<Color>,
    border_width: u32,
    radius: u32
}

impl Frame {
    fn new(position: Position) -> Self {
        Frame {
            pos: position,
            fill: None,
            border: None,
            border_width: 0,
            radius: 0
        }
    }

    fn empty() -> Self {
        Frame {
            pos: Position { upper_left: Point::origin(), size: Size { w: 0, h: 0 } },
            fill: None,
            border: None,
            border_width: 0,
            radius: 0
        }
    }

    pub(crate) fn color(&self) -> &Option<Color> {
        &self.fill
    }
}

pub(crate) struct ListLayout {
    children: Vec<ListItem>,
    position: Position,
    pub(crate) item_size: Size,
    spacing: u32,
    selected: usize
}

impl ListLayout {
    pub(crate) fn new(item_size: Size) -> Self {
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

    pub fn add_item(&mut self, mut item: ListItem) {
        item.set_position(Position {
            upper_left: Point {
                x: self.position.size.w as i32 + self.spacing as i32,
                y: 0
            },
            size: self.item_size.clone()
        });
        self.position.size.w += (self.spacing + self.item_size.w);
        self.children.push(item);
    }

    fn select_next(&mut self) {
        if self.selected < self.children.len() - 1 {
            self.selected += 1;
            self.position.upper_left.translate(0 - (self.item_size.w + self.spacing) as i32, 0)
        }
    }

    fn select_prev(&mut self) {
        if self.selected > 0 {
            self.selected -= 1;
            self.position.upper_left.translate((self.item_size.w + self.spacing) as i32, 0)
        }
    }
}

pub struct ListItem {
    item: LayoutItem,
    selected_item: LayoutItem,
    pos: Position
}

impl ListItem {
    fn new(unselected: LayoutItem, selected: LayoutItem) -> Self {
        ListItem {
            item: unselected,
            selected_item: selected,
            pos: Position::new(Point::origin(), Size::new(0, 0))
        }
    }
}

pub struct GameItemFactory;

impl GameItemFactory {
    pub(crate) fn make(model: crate::data::GameModel) -> ListItem {
        ListItem::new(GameItemFactory::make_item(&model), GameItemFactory::make_selected_item(&model))
    }

    fn make_item(model: &crate::data::GameModel) -> LayoutItem {
        let thumbnail = Image::from_bytes(
            model.image.clone(),
            Position::new(Point::origin(),
            Size::new(model.image_w, model.image_h)));


        let mut vb_layout = VBoxLayout::new();
        vb_layout.add_child(LayoutItem::Widget(WidgetType::Image(thumbnail)));

        let mut vc_layout = VCenteredLayout::new(Position::new(Point::origin(), Size::new(200, 300)));
        vc_layout.add_child(LayoutItem::Layout(Box::new(vb_layout)));

        LayoutItem::Layout(Box::new(vc_layout))
    }

    fn make_selected_item(model: &crate::data::GameModel) -> LayoutItem {
        let home_team = Text {
            content: model.home_team.clone(),
            size: 18,
            pos: Position { upper_left: Point::origin(), size: Size::new(0, 30) },
            color: Color::new(255, 255, 255)
        };

        let vs = Text {
            content: "VS".to_owned(),
            size: 12,
            pos: Position { upper_left: Point::origin(), size: Size::new(0, 10) },
            color: Color::new(255, 255, 255)
        };

        let away_team = Text {
            content: model.away_team.clone(),
            size: 18,
            pos: Position { upper_left: Point::origin(), size: Size::new(0, 30) },
            color: Color::new(255, 255, 255)
        };

        let thumbnail = Image::from_bytes(
            model.image.clone(),
            Position::new(Point::origin(),
                          Size::new(model.image_w, model.image_h)));

        let desc = Text {
            content: model.description.clone(),
            size: 12,
            pos: Position::new( Point::origin(), Size::new(0, 30)),
            color: Color::new(255, 255, 255)
        };

        let mut vb_layout = VBoxLayout::new();
        vb_layout.add_child(LayoutItem::Widget(WidgetType::Text(home_team)));
        vb_layout.add_child(LayoutItem::Widget(WidgetType::Text(vs)));
        vb_layout.add_child(LayoutItem::Widget(WidgetType::Text(away_team)));
        vb_layout.add_child(LayoutItem::Widget(WidgetType::Image(thumbnail)));
        vb_layout.add_child(LayoutItem::Widget(WidgetType::Text(desc)));

        let mut vc_layout = VCenteredLayout::new(Position::new(Point::origin(), Size::new(200, 300)));
        vc_layout.add_child(LayoutItem::Layout(Box::new(vb_layout)));

        LayoutItem::Layout(Box::new(vc_layout))
    }
}

pub struct VBoxLayout {
    children: Vec<LayoutItem>,
    position: Position
}

impl VBoxLayout {
    pub fn new() -> Self {
        VBoxLayout {
            children: vec![],
            position: Position::new(Point::origin(), Size::new(0, 0))
        }
    }

    pub fn add_child(&mut self, item: LayoutItem) {
        self.children.push(item);
        self.position_children();
    }

    fn position_children(&mut self) {
        let mut h: i32 = 0;
        for child in &mut self.children {
            let mut new_pos = child.position().clone();
            new_pos.size.w = self.position.size.w;
            new_pos.upper_left = Point::new(0, h);
            child.set_position(new_pos);
            h += child.position().size.h as i32;
        }
        self.position.size.h = h as u32;
    }
}

impl Layout for VBoxLayout {
    fn child_at(&mut self, index: usize) -> Option<&mut LayoutItem> {
        self.children.get_mut(index)
    }
}

impl Responsive for VBoxLayout {
    fn handle_key(&mut self, key: Key) -> bool { false }
}

pub struct CenteredLayout {
    children: Vec<LayoutItem>,
    position: Position,
    center: Point
}

impl CenteredLayout {
    pub(crate) fn new(position: Position) -> Self {
        CenteredLayout {
            children: vec![],
            center: Point{x: position.upper_left.x + (position.size.w / 2) as i32, y: position.upper_left.y + (position.size.h / 2) as i32},
            position
        }
    }

    pub(crate) fn add_child(&mut self, mut item: LayoutItem) {
        let new_pos = Position {
            upper_left: Point { x: self.center.x - (item.position().size.w / 2) as i32, y: self.center.y - (item.position().size.h / 2) as i32 },
            size: item.position().size.clone()
        };
        item.set_position(new_pos);
        println!("centered layout child pos: {:?}", item.position());
        self.children.push(item);
    }
}

pub(crate) struct VCenteredLayout {
    children: Vec<LayoutItem>,
    position: Position,
}

impl VCenteredLayout {
    pub(crate) fn new(position: Position) -> Self {
        VCenteredLayout {
            children: vec![],
            position
        }
    }

    pub(crate) fn add_child(&mut self, mut item: LayoutItem) {
        item.set_position(VCenteredLayout::calculate_position(self.position(), item.position()));
        self.children.push(item);
    }

    fn calculate_position(layout: &Position, item: &Position) -> Position {
        Position {
            upper_left: Point { x: item.upper_left.x, y: layout.center().y - (item.size.h / 2) as i32},
            size: Size::new(layout.size.w, item.size.h)
        }
    }

    fn position_children(&mut self) {
        let position = self.position().clone();
        for child in &mut self.children {
            child.set_position(VCenteredLayout::calculate_position(&position, child.position()));
        }
    }
}

pub enum WidgetType {
    Frame(Frame),
    Image(Image),
    Text(Text)
}

pub enum LayoutItem {
    Layout(Box<dyn Layout>),
    Widget(WidgetType)
}

pub trait Positionable {
    fn set_position(&mut self, pos: Position);
    fn position(&self) -> &Position;
}

// impl Positionable for Box<dyn Layout> {
//     fn set_position(&mut self, pos: Position) {
//         println!("setting layout position: {:?}", pos);
//
//     }
//
//     fn position(&self) -> &Position {
//
//     }
// }

impl Positionable for VBoxLayout {
    fn set_position(&mut self, pos: Position) {
        println!("Positioning vb layout: {:?}", pos);
        self.position = pos;
        self.position_children();
    }

    fn position(&self) -> &Position {
        &self.position
    }
}

impl Positionable for Frame {
    fn set_position(&mut self, pos: Position) {
        self.pos = pos;
    }

    fn position(&self) -> &Position {
        &self.pos
    }
}

impl Positionable for Text {
    fn set_position(&mut self, pos: Position) {
        self.pos = pos;
    }

    fn position(&self) -> &Position {
        &self.pos
    }
}

impl Positionable for Image {
    fn set_position(&mut self, pos: Position) {
        println!("Positioning Image: {:?}", pos);
        if self.preserve_aspect_w {
            let aspect = 16.0 / 9.0;
            let new_size = Size::new(pos.size.w, (pos.size.w as f32 / aspect) as u32);
            println!("new size: {:?}", new_size);
            self.pos = Position::new(pos.upper_left, new_size);
        } else {
            self.pos = pos;
        }
    }

    fn position(&self) -> &Position {
        &self.pos
    }
}

impl Positionable for WidgetType {
    fn set_position(&mut self, pos: Position) {
        match self {
            WidgetType::Frame(f) => { f.set_position(pos) }
            WidgetType::Image(i) => { i.set_position(pos) }
            WidgetType::Text(t) => { t.set_position(pos) }
        }
    }

    fn position(&self) -> &Position {
        match self {
            WidgetType::Frame(f) => { f.position() }
            WidgetType::Image(i) => { i.position() }
            WidgetType::Text(t) => { t.position() }
        }
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

impl Positionable for ListItem {
    fn set_position(&mut self, pos: Position) {
        let mut scaled_pos = pos.clone();
        let center = pos.center();
        scaled_pos.size = Size::new((pos.size.w as f32 * 0.75) as u32, (pos.size.h as f32 * 0.75) as u32);
        scaled_pos.upper_left.x = center.x - (scaled_pos.size.w / 2) as i32;
        scaled_pos.upper_left.y = center.y - (scaled_pos.size.h / 2) as i32;
        self.item.set_position(scaled_pos);
        self.selected_item.set_position(pos);
    }

    fn position(&self) -> &Position {
        self.item.position()
    }
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
        println!("Positioning vc layout: {:?}", pos);
        self.position = pos;
        self.position_children();
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

pub trait Layout: Positionable + Responsive {
    fn child_at(&mut self, index: usize) -> Option<&mut LayoutItem>;
}

impl Layout for ListItem {
    fn child_at(&mut self, _: usize) -> Option<&mut LayoutItem> {
        unimplemented!()
    }
}

impl Layout for CenteredLayout {
    fn child_at(&mut self, index: usize) -> Option<&mut LayoutItem> {
        self.children.get_mut(index)
    }
}

impl Layout for VCenteredLayout {
    fn child_at(&mut self, index: usize) -> Option<&mut LayoutItem> {
        self.children.get_mut(index)
    }
}

impl Layout for ListLayout {
    fn child_at(&mut self, index: usize) -> Option<&mut LayoutItem> {
        match self.children.get_mut(index) {
            None => None,
            Some(item) => match index == self.selected {
                true => Some(&mut item.selected_item),
                false => Some(&mut item.item)
            }
        }
    }
}

pub trait Responsive {
    fn handle_key(&mut self, key: Key) -> bool;
}

impl Responsive for ListItem {
    fn handle_key(&mut self, key: Key) -> bool {
        false
    }
}

impl Responsive for ListLayout {
    fn handle_key(&mut self, key: Key) -> bool {
        match key {
            Key::Right => { self.select_next(); }
            Key::Left => { self.select_prev(); }
        }
        true
    }
}

impl Responsive for CenteredLayout {
    fn handle_key(&mut self, key: Key) -> bool {
        false
    }
}

impl Responsive for VCenteredLayout {
    fn handle_key(&mut self, key: Key) -> bool {
        false
    }
}

pub(crate) fn translate_to_global(relative: &Point, global: &Point) -> Point {
    Point {
        x: global.x + relative.x,
        y: global.y + relative.y
    }
}

pub(crate) fn translate_to_relative(global: &Point, relative: &Point) -> Point {
    Point {
        x: global.x - relative.x,
        y: global.y - relative.y
    }
}

pub trait Visitee<T> {
    fn accept_visitor(&mut self, visitor: &mut dyn Visitor<T>);
}

impl<T> Visitee<T> for T {
    fn accept_visitor(&mut self, visitor: &mut dyn Visitor<T>) {
        visitor.visit_element(self);
    }
}

pub trait Visitor<T> {
    fn visit_element(&mut self, element: &mut T);
}

trait Widget: Positionable {
    fn get_type(&self) -> WidgetType;
}