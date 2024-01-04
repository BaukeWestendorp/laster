#![feature(array_chunks)]

use std::io::Read;
use std::rc::Rc;

use dom::arena::NodeArena;
use dom::node::{Node, NodeKind};
use fleck::Font;
use pixels::wgpu::BlendState;
use pixels::{PixelsBuilder, SurfaceTexture};
use stammer::elements::builder::ElementBuilder;
use stammer::elements::{Element, SizingStrategy};
use stammer::Panel;
use winit::dpi::{LogicalSize, PhysicalSize};
use winit::event::Event;
use winit::event_loop::EventLoop;
use winit::window::{Window, WindowBuilder};
use winit_input_helper::WinitInputHelper;

const WINDOW_NAME: &str = env!("CARGO_BIN_NAME");

fn setup_window(min_size: PhysicalSize<u32>, event_loop: &EventLoop<()>) -> Window {
    let builder = WindowBuilder::new()
        .with_transparent(true)
        .with_resizable(true)
        .with_title(WINDOW_NAME)
        .with_inner_size(min_size)
        .with_min_inner_size(min_size);
    builder.build(&event_loop).expect("should build window")
}

fn load_font(path: &str) -> std::io::Result<Font> {
    let mut file = std::fs::File::open(path)?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;
    assert_eq!(buffer.len(), fleck::FILE_SIZE);
    let font = Font::new(buffer.as_slice().try_into().unwrap());
    Ok(font)
}

fn dom_node_as_stammer_element(
    font: Rc<Font>,
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

fn main() -> Result<(), pixels::Error> {
    let mut arena = dom::arena::NodeArena::new();
    let document = get_document(&mut arena);
    let body = get_body(&mut arena, &document);

    let font_path = "/etc/tid/fonts/geneva14.uf2".to_string();
    let font = match load_font(&font_path) {
        Ok(font) => font,
        Err(err) => {
            eprintln!("ERROR: Failed to load font from {font_path:?}: {err}");
            std::process::exit(1);
        }
    };

    let event_loop = EventLoop::new();

    let scale_factor = std::env::var("TID_SCALE_FACTOR")
        .ok()
        .and_then(|v| v.parse::<f32>().ok())
        .map(|v| v.round() as u32)
        .unwrap_or(1);

    let mut document_element = dom_node_as_stammer_element(Rc::new(font), &body, &mut arena);
    document_element.size.maxwidth = Some(600);
    document_element.size.maxheight = Some(400);

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
    // TODO: This is _SUCH_ a papercut or even pitfall, as I just saw.
    state.data_mut().width = width;
    state.data_mut().height = height;
    let size = PhysicalSize::new(width * scale_factor, height * scale_factor);

    let mut input = WinitInputHelper::new();
    let window = setup_window(size, &event_loop);

    let mut pixels = {
        let window_size = window.inner_size();
        let surface_texture = SurfaceTexture::new(window_size.width, window_size.height, &window);
        PixelsBuilder::new(width, height, surface_texture)
            .clear_color({
                let [r, g, b, a] = state.background.map(|v| v as f64 / u8::MAX as f64);
                pixels::wgpu::Color { r, g, b, a }
            })
            .blend_state(BlendState::REPLACE) // TODO: Investigate rendering weirdness.
            .build()?
    };

    event_loop.run(move |event, _, control_flow| {
        control_flow.set_poll();

        match event {
            // Event::NewEvents(winit::event::StartCause::ResumeTimeReached { .. }) => {
            //     window.request_redraw()
            // }
            Event::RedrawRequested(_) => {
                // Clear the screen before drawing.
                pixels
                    .frame_mut()
                    .array_chunks_mut()
                    .for_each(|px| *px = state.background);

                // Update the state, then draw.
                state.update();
                state.draw(&mut pixels.frame_mut());

                // Try to render.
                if let Err(err) = pixels.render() {
                    eprintln!("ERROR: {err}");
                    control_flow.set_exit();
                    return;
                }
            }
            _ => (),
        }

        if input.update(&event) {
            // Close events.
            if input.close_requested() {
                eprintln!("INFO:  Close requested. Bye :)");
                control_flow.set_exit();
                return;
            }

            // Resize the window.
            if let Some(size) = input.window_resized() {
                eprintln!("INFO:  Resize request {size:?}");
                let ps = PhysicalSize {
                    width: (size.width / scale_factor) * scale_factor,
                    height: (size.height / scale_factor) * scale_factor,
                };
                let ls = LogicalSize {
                    width: ps.width / scale_factor,
                    height: ps.height / scale_factor,
                };
                pixels.resize_surface(ps.width, ps.height).unwrap();
                pixels.resize_buffer(ls.width, ls.height).unwrap();
                window.set_inner_size(ps);
                state.resize(ls.width, ls.height);
                state.data_mut().width = ls.width;
                state.data_mut().height = ls.height;
                window.request_redraw();
            }
        }
    });
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
