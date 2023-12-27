use arena::NodeArena;
use node::Node;

pub mod arena;
pub mod node;
mod parser;
mod tokenizer;

#[derive(Debug, Clone, PartialEq)]
pub struct Dom {}

impl Dom {
    pub fn parse(html: &str, arena: &mut NodeArena) -> Node {
        let document = parser::Parser::new(html, arena).parse();
        document
    }

    pub fn parse_file(path: &str, arena: &mut NodeArena) -> Node {
        let file_content = std::fs::read_to_string(path).unwrap();
        Dom::parse(&file_content, arena)
    }
}
