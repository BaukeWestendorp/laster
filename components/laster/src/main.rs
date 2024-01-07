#![feature(array_chunks)]

use nannou::prelude::*;

use std::io::Read;
use std::rc::Rc;

use dom::arena::NodeArena;
use dom::node::{Node, NodeKind};
use stammer::elements::builder::ElementBuilder;
use stammer::elements::{Element, SizingStrategy};
use stammer::Panel;

fn dom_node_as_stammer_element(
    font: Rc<stammer::Font>,
    node: &Node,
    arena: &mut NodeArena,
) -> Element<Data> {
    let mut children = vec![];
    for child in node.children().iter() {
        let child = arena.get_node(*child).clone();

        let element = match child.kind {
            NodeKind::Text { data } => {
                if data.trim().is_empty() {
                    continue;
                }

                Element::paragraph(data.as_str(), &font)
                    .build()
                    .with_maxwidth(400)
            }
            _ => dom_node_as_stammer_element(font.clone(), &child, arena),
        };
        children.push(element);
    }
    Element::stack_builder(&font)
        .add_children(children.into_iter())
        .build()
        .with_strategy(SizingStrategy::Chonker)
}

struct Data {
    width: u32,
    height: u32,
}

fn get_document(arena: &mut NodeArena) -> Node {
    let mut html = String::new();
    std::io::stdin().read_to_string(&mut html).unwrap();

    let document = dom::Dom::parse(html.as_str(), arena);

    document.dump(&arena);

    document
}

fn get_body(arena: &mut NodeArena, document: &Node) -> Node {
    let html = document.children()[1];
    let head = arena.get_node(html).children()[0];
    let body = arena.get_node(head).children()[7];
    let body = arena.get_node(body).clone();
    body
}

fn main() {
    nannou::app(model)
        .event(event)
        .loop_mode(LoopMode::Wait)
        .run();
}

struct Model {
    texture: wgpu::Texture,
}

fn model(app: &App) -> Model {
    let mut arena = dom::arena::NodeArena::new();
    let document = get_document(&mut arena);
    let body = get_body(&mut arena, &document);

    let font_path = "/etc/tid/fonts/times15.uf2".to_string();
    let font = match stammer::Font::load_from_file(&font_path) {
        Ok(font) => font,
        Err(err) => {
            eprintln!("ERROR: Failed to load font from {font_path:?}: {err}");
            std::process::exit(1);
        }
    };

    let mut document_element = dom_node_as_stammer_element(Rc::new(font), &body, &mut arena);
    document_element.size.maxwidth = Some(512);
    document_element.size.maxheight = Some(512);

    let data = Data {
        width: 0,
        height: 0,
    };

    let mut state = Panel::new(
        document_element,
        [0x00, 0x00, 0x00, 0xff],
        [0xff, 0xff, 0xff, 0xff],
        data,
    );

    let (width, height) = (state.width, state.height);
    state.data_mut().width = width;
    state.data_mut().height = height;

    let window = app.new_window().size(512, 512).view(view).build().unwrap();
    let window = app.window(window).unwrap();

    let texture = stammer_nannou::panel_to_texture(&state, &window);

    Model { texture }
}

fn event(_app: &App, _model: &mut Model, _event: Event) {}

fn view(app: &App, model: &Model, frame: Frame) {
    let draw = app.draw();
    draw.texture(&model.texture);
    draw.to_frame(app, &frame).unwrap();
}
