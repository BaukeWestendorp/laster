use node::Node;

mod arena;
pub mod node;
mod parser;
mod tokenizer;

#[derive(Debug, Clone, PartialEq)]
pub struct Dom {}

impl Dom {
    pub fn parse(html: &str) -> Node {
        let document = parser::Parser::new(html).parse();
        document
    }

    pub fn parse_file(path: &str) -> Node {
        let file_content = std::fs::read_to_string(path).unwrap();
        Dom::parse(&file_content)
    }
}
