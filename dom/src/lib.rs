mod parser;
mod tokenizer;

#[derive(Debug, Clone, PartialEq)]
pub struct Dom {}

impl Dom {
    pub fn parse(html: &str) -> Dom {
        let tokens = tokenizer::Tokenizer::new(html).tokenize();
        let dom = parser::Parser::new(tokens).parse();
        dom
    }

    pub fn parse_file(path: &str) -> Dom {
        let file_content = std::fs::read_to_string(path).unwrap();
        Dom::parse(&file_content)
    }
}
