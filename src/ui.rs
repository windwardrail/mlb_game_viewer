pub mod layouts;

use std::path::{PathBuf};

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

#[derive(Debug)]
pub struct Image {
    pub(crate) source: PathBuf,
    pos: Position
}

impl Image {
    pub fn new(source: PathBuf, pos: Position) -> Self {
        Image { source, pos }
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
        println!("item pos: {:?}", item.position());
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

pub(crate) struct ListItemFactory {}

impl ListItemFactory {
    pub(crate) fn make() -> ListItem {
        let mut item = Frame::empty();
        item.fill = Some(Color::new(255, 0, 0 ));
        let mut selected = Frame::empty();
        selected.fill = Some(Color::new(0, 255, 0));
        ListItem::new(LayoutItem::Widget(WidgetType::Frame(item)), LayoutItem::Widget(WidgetType::Frame(selected)))
    }
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
        let item_pos = item.position();
        let new_pos = Position {
            upper_left: Point { x: item_pos.upper_left.x, y: self.position.center().y - (item_pos.size.h / 2) as i32},
            size: item_pos.size.clone()
        };
        item.set_position(new_pos);
        self.children.push(item);
    }
}

pub(crate) enum WidgetType {
    Frame(Frame),
    Image(Image),
    Text(Text)
}

pub(crate) enum LayoutItem {
    Layout(Box<dyn Layout>),
    Widget(WidgetType)
}

pub trait Positionable {
    fn set_position(&mut self, pos: Position);
    fn position(&self) -> &Position;
}

impl Positionable for Frame {
    fn set_position(&mut self, pos: Position) {
        self.pos = pos;
    }

    fn position(&self) -> &Position {
        &self.pos
    }
}

impl Positionable for Image {
    fn set_position(&mut self, pos: Position) {
        self.pos = pos;
    }

    fn position(&self) -> &Position {
        &self.pos
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
        self.item.set_position(pos.clone());
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

pub(crate) trait Layout: Positionable + Responsive {
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