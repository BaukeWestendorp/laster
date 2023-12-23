use crate::arena::{NodeArena, NodeId};
use crate::tokenizer::{self, Token};
use crate::Dom;

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum InsertionMode {
    Initial,
    BeforeHtml,
    BeforeHead,
    InHead,
    InHeadNoScript,
    AfterHead,
    InBody,
    Text,
    InTable,
    InTableText,
    InCaption,
    InColumnGroup,
    InTableBody,
    InRow,
    InCell,
    InSelect,
    InSelectInTable,
    InTemplate,
    AfterBody,
    InFrameset,
    AfterFrameset,
    AfterAfterBody,
    AfterAfterFrameset,
}

#[derive(Debug, Clone)]
pub struct Parser<'input> {
    arena: NodeArena,
    tokenizer: tokenizer::Tokenizer<'input>,
    insertion_mode: InsertionMode,
    should_reprocess_token: bool,
    open_elements: Vec<NodeId>,
}

impl<'input> Parser<'input> {
    pub fn new(html: &'input str) -> Self {
        Self {
            arena: NodeArena::new(),
            tokenizer: tokenizer::Tokenizer::new(html),
            insertion_mode: InsertionMode::Initial,
            should_reprocess_token: false,
            open_elements: vec![],
        }
    }

    pub fn parse(mut self) -> Dom {
        while let Some(token) = match self.should_reprocess_token {
            true => self.tokenizer.peek().cloned(),
            false => self.tokenizer.next(),
        } {
            self.should_reprocess_token = false;
            self.dispatch(&token)
        }
        Dom {}
    }

    /// https://html.spec.whatwg.org/multipage/parsing.html#tree-construction-dispatcher
    fn dispatch(&mut self, token: &Token) {
        if !self.is_in_foreign_content(&token) {
            self.process_token(token);
        } else {
            todo!("Implement foreign content parsing algorithm");
        }
    }

    fn process_token(&mut self, token: &Token) {
        macro_rules! whitespace {
            () => {
                Token::Character('\u{0009}')
                    | Token::Character('\u{000A}')
                    | Token::Character('\u{000C}')
                    | Token::Character('\u{000D}')
                    | Token::Character('\u{0020}')
            };
        }

        match self.insertion_mode {
            InsertionMode::Initial => match token {
                whitespace!() => {}
                Token::Comment => {
                    todo!("Insert a comment as the last child of the Document object.")
                }
                Token::Doctype => {
                    todo!("Implement DOCTYPE token parsing in initial insertion mode")
                }
                _ => {
                    // TODO: If the document is not an iframe srcdoc document, then this is a parse
                    // error; if the parser cannot change the mode flag is false, set the Document
                    // to quirks mode.
                    self.switch_insertion_mode(InsertionMode::BeforeHtml);
                }
            },
            InsertionMode::BeforeHtml => match token {
                Token::Doctype => {
                    todo!("Parse error. Ignore the token.");
                }
                Token::Comment => {
                    todo!("Insert a comment as the last child of the Document object.")
                }
                whitespace!() => {}
                Token::Tag { .. } if token.is_start_tag_with_name(&["html"]) => {
                    todo!()
                }
                Token::Tag { .. }
                    if token.is_end_tag_with_name(&["head", "body", "html", "br"]) =>
                {
                    todo!("Act as described in the 'anything else' entry below.")
                }
                Token::Tag {
                    is_start_tag: false,
                    ..
                } => {
                    todo!("Parser error. Ignore the token.")
                }
                _ => {
                    // TODO: Create an html element whose node document is the Document object.
                    // Append it to the Document object. Put this element in the stack of open
                    // elements.

                    self.switch_insertion_mode_and_reprocess_token(InsertionMode::BeforeHead);
                }
            },
            InsertionMode::BeforeHead => todo!("BeforeHead"),
            InsertionMode::InHead => todo!("InHead"),
            InsertionMode::InHeadNoScript => todo!("InHeadNoScript"),
            InsertionMode::AfterHead => todo!("AfterHead"),
            InsertionMode::InBody => todo!("InBody"),
            InsertionMode::Text => todo!("Text"),
            InsertionMode::InTable => todo!("InTable"),
            InsertionMode::InTableText => todo!("InTableText"),
            InsertionMode::InCaption => todo!("InCaption"),
            InsertionMode::InColumnGroup => todo!("InColumnGroup"),
            InsertionMode::InTableBody => todo!("InTableBody"),
            InsertionMode::InRow => todo!("InRow"),
            InsertionMode::InCell => todo!("InCell"),
            InsertionMode::InSelect => todo!("InSelect"),
            InsertionMode::InSelectInTable => todo!("InSelectInTable"),
            InsertionMode::InTemplate => todo!("InTemplate"),
            InsertionMode::AfterBody => todo!("AfterBody"),
            InsertionMode::InFrameset => todo!("InFrameset"),
            InsertionMode::AfterFrameset => todo!("AfterFrameset"),
            InsertionMode::AfterAfterBody => todo!("AfterAfterBody"),
            InsertionMode::AfterAfterFrameset => todo!("AfterAfterFrameset"),
        }
    }

    fn switch_insertion_mode(&mut self, insertion_mode: InsertionMode) {
        self.insertion_mode = insertion_mode;
    }

    fn switch_insertion_mode_and_reprocess_token(&mut self, insertion_mode: InsertionMode) {
        self.should_reprocess_token = true;
        self.switch_insertion_mode(insertion_mode);
    }

    fn stack_of_open_elements_is_empty(&self) -> bool {
        self.open_elements.len() == 0
    }

    fn current_node(&self) -> NodeId {
        *self
            .open_elements
            .last()
            .expect("Should always have a value. If not the parser should have finished.")
    }

    fn adjusted_current_node(&self) -> NodeId {
        // TODO: The adjusted current node is the context element
        //      if the parser was created as part of the
        //      HTML fragment parsing algorithm and the stack of open elements
        //      has only one element in it (fragment case);

        // otherwise, the adjusted current node is the current node.
        self.current_node()
    }

    fn is_in_foreign_content(&self, token: &Token) -> bool {
        !(self.stack_of_open_elements_is_empty() ||
        // TODO: If the adjusted current node is an element in the HTML namespace
        // TODO: If the adjusted current node is a MathML text integration point and the token is a start tag whose tag name is neither "mglyph" nor "malignmark"
        // TODO: If the adjusted current node is a MathML text integration point and the token is a character token
        // TODO: If the adjusted current node is a MathML annotation-xml element and the token is a start tag whose tag name is "svg"
        // TODO: If the adjusted current node is an HTML integration point and the token is a start tag
        // TODO: If the adjusted current node is an HTML integration point and the token is a character token
        token == &Token::EndOfFile)
    }
}
