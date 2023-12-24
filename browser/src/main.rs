fn main() {
    let html = r#"<html><head></head><body><p>Hello, world!</p></body></html>"#;
    let dom = dom::Dom::parse(html);
    dbg!(dom);
}
