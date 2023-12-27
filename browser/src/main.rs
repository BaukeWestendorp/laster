fn main() {
    let html = r#"<html><head></head><body><p>Hello, world!</p></body></html>"#;
    let mut arena = dom::arena::NodeArena::new();
    let document = dom::Dom::parse(html, &mut arena);

    dbg!(document);
}
