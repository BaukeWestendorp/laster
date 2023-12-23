fn main() {
    let html = r#"<html><head></head><body><h1 class="title">Hello, world!</h1></body></html>"#;
    let dom = dom::Dom::parse(html);
    dbg!(dom);
}
