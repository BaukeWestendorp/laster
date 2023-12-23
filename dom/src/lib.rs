mod arena;
pub mod node;
mod parser;
mod tokenizer;

#[derive(Debug, Clone, PartialEq)]
pub struct Dom {}

impl Dom {
    pub fn parse(html: &str) -> Dom {
        let dom = parser::Parser::new(html).parse();
        dom
    }

    pub fn parse_file(path: &str) -> Dom {
        let file_content = std::fs::read_to_string(path).unwrap();
        Dom::parse(&file_content)
    }
}
