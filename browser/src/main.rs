use std::io::Read;

fn main() {
    let mut html = String::new();
    std::io::stdin().read_to_string(&mut html).unwrap();

    let mut arena = dom::arena::NodeArena::new();
    let document = dom::Dom::parse(html.as_str(), &mut arena);

    document.dump(&arena);
}
