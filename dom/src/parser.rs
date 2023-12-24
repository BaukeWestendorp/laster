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
    should_stop_parsing: bool,
    scripting: bool,
}

impl<'input> Parser<'input> {
    pub fn new(html: &'input str) -> Self {
        Self {
            arena: NodeArena::new(),
            tokenizer: tokenizer::Tokenizer::new(html),
            insertion_mode: InsertionMode::Initial,
            should_reprocess_token: false,
            open_elements: vec![],
            should_stop_parsing: false,
            scripting: false,
        }
    }

    pub fn parse(mut self) -> Dom {
        while let Some(token) = match self.should_reprocess_token {
            true => self.tokenizer.peek().cloned(),
            false => self.tokenizer.next(),
        } {
            if self.should_stop_parsing {
                break;
            }

            self.should_reprocess_token = false;
            self.dispatch(&token)
        }
        Dom {}
    }

    /// https://html.spec.whatwg.org/multipage/parsing.html#tree-construction-dispatcher
    fn dispatch(&mut self, token: &Token) {
        if !self.is_in_foreign_content(&token) {
            self.process_token(self.insertion_mode, token);
        } else {
            todo!("Implement foreign content parsing algorithm");
        }
    }

    fn process_token(&mut self, insertion_mode: InsertionMode, token: &Token) {
        macro_rules! whitespace {
            () => {
                Token::Character('\u{0009}')
                    | Token::Character('\u{000A}')
                    | Token::Character('\u{000C}')
                    | Token::Character('\u{000D}')
                    | Token::Character('\u{0020}')
            };
        }

        match insertion_mode {
            InsertionMode::Initial => match token {
                whitespace!() => {}
                Token::Comment => {
                    todo!("Insert a comment as the last child of the Document object.");
                }
                Token::Doctype => {
                    todo!("Implement DOCTYPE token parsing in initial insertion mode");
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
                    todo!("Insert a comment as the last child of the Document object.");
                }
                whitespace!() => {}
                Token::Tag { .. } if token.is_start_tag_with_name(&["html"]) => {
                    todo!();
                }
                Token::Tag { .. }
                    if token.is_end_tag_with_name(&["head", "body", "html", "br"]) =>
                {
                    todo!("Act as described in the 'anything else' entry below.");
                }
                Token::Tag { .. } if token.is_end_tag() => {
                    todo!("Parser error. Ignore the token.");
                }
                _ => {
                    // TODO: Create an html element whose node document is the Document object.
                    // Append it to the Document object. Put this element in the stack of open
                    // elements.

                    self.switch_insertion_mode_and_reprocess_token(InsertionMode::BeforeHead);
                }
            },
            InsertionMode::BeforeHead => match token {
                whitespace!() => {}
                Token::Comment => {
                    todo!("Insert a comment.");
                }
                Token::Doctype => {
                    todo!("Parse error. Ignore the token.");
                }
                Token::Tag { .. } if token.is_start_tag_with_name(&["html"]) => {
                    todo!("Process the token using the rules for the 'in body' insertion mode.");
                }
                Token::Tag { .. } if token.is_start_tag_with_name(&["head"]) => {
                    // TODO: Insert an HTML element for the token.

                    // TODO: Set the head element pointer to the newly created
                    // head element.

                    // TODO: Switch the insertion mode to "in head".
                    self.switch_insertion_mode(InsertionMode::InHead);
                }
                Token::Tag { .. }
                    if token.is_end_tag_with_name(&["head", "body", "html", "br"]) =>
                {
                    todo!("Act as described in the 'anything else' entry below.");
                }
                Token::Tag { .. } if token.is_end_tag() => {
                    todo!("Parse error. Ignore the token.");
                }
                _ => {
                    todo!();
                }
            },
            InsertionMode::InHead => match token {
                whitespace!() => {
                    todo!("Insert the character");
                }
                Token::Comment => {
                    todo!("Insert a comment.");
                }
                Token::Doctype => {
                    todo!("Parse error. Ignore the token.");
                }
                Token::Tag { .. } if token.is_start_tag_with_name(&["html"]) => {
                    self.process_token(InsertionMode::InBody, token);
                }
                Token::Tag { .. }
                    if token.is_start_tag_with_name(&["base", "basefont", "bgsound", "link"]) =>
                {
                    // TODO: Insert an HTML element for the token. Immediately pop the current node
                    // off the stack of open elements.

                    // TODO: Acknowledge the token's self-closing flag, if it is set.

                    todo!();
                }
                Token::Tag { .. } if token.is_start_tag_with_name(&["meta"]) => {
                    todo!();
                }
                Token::Tag { .. } if token.is_start_tag_with_name(&["title"]) => {
                    todo!("Follow the generic RCDATA element parsing algorithm.");
                }
                Token::Tag { .. }
                    if (token.is_start_tag_with_name(&["noscript"]) && self.scripting)
                        || token.is_start_tag_with_name(&["noframes", "style"]) =>
                {
                    todo!("Follow the generic raw text element parsing algorithm.");
                }
                Token::Tag { .. } if token.is_start_tag_with_name(&["script"]) => {
                    todo!();
                }
                Token::Tag { .. } if token.is_end_tag_with_name(&["head"]) => {
                    // TODO: Pop the current node (which will be the head element) off the stack of
                    // open elements.

                    self.switch_insertion_mode(InsertionMode::AfterHead);
                }
                Token::Tag { .. } if token.is_end_tag_with_name(&["body", "html", "br"]) => {
                    todo!("Act as described in the 'anything else' entry below.");
                }
                Token::Tag { .. } if token.is_start_tag_with_name(&["template"]) => {
                    todo!();
                }
                Token::Tag { .. } if token.is_end_tag_with_name(&["template"]) => {
                    todo!();
                }
                Token::Tag { .. }
                    if token.is_start_tag_with_name(&["head"]) || token.is_end_tag() =>
                {
                    todo!("Parse error. Ignore the token.");
                }
                _ => {
                    // TODO: Pop the current node (which will be the head element) off the stack of
                    // open elements.

                    self.switch_insertion_mode_and_reprocess_token(InsertionMode::AfterHead);
                }
            },
            InsertionMode::InHeadNoScript => todo!("InHeadNoScript"),
            InsertionMode::AfterHead => match token {
                whitespace!() => {
                    todo!("Insert the character.");
                }
                Token::Comment => {
                    todo!("Insert a comment.");
                }
                Token::Doctype => {
                    todo!("Parse error. Ignore the token.");
                }
                Token::Tag { .. } if token.is_start_tag_with_name(&["html"]) => {
                    self.process_token(InsertionMode::InBody, token)
                }
                Token::Tag { .. } if token.is_start_tag_with_name(&["body"]) => {
                    // TODO: Insert an HTML element for the token.

                    // TODO: Set the frameset-ok flag to "not ok".

                    self.switch_insertion_mode(InsertionMode::InBody);
                }
                Token::Tag { .. } if token.is_start_tag_with_name(&["frameset"]) => {
                    // TODO: Insert an HTML element for the token.

                    self.switch_insertion_mode(InsertionMode::InFrameset);
                }
                Token::Tag { .. }
                    if token.is_start_tag_with_name(&[
                        "base", "basefont", "bgsound", "link", "meta", "noframes", "script",
                        "style", "template", "title",
                    ]) =>
                {
                    todo!();
                }
                Token::Tag { .. } if token.is_end_tag_with_name(&["template"]) => {
                    self.process_token(InsertionMode::InHead, token);
                }
                Token::Tag { .. } if token.is_end_tag_with_name(&["body", "html", "br"]) => {
                    todo!("Act as described in the 'anything else' entry below.")
                }
                Token::Tag { .. }
                    if token.is_start_tag_with_name(&["head"]) || token.is_end_tag() =>
                {
                    todo!("Parse error. Ignore the token.")
                }
                _ => {
                    // TODO: Insert an HTML element for a "body" start tag token with no attributes.

                    self.switch_insertion_mode_and_reprocess_token(InsertionMode::InBody);
                }
            },
            InsertionMode::InBody => match token {
                Token::Character('\0') => todo!(),
                whitespace!() => todo!(),
                Token::Character(_) => todo!(),
                Token::Comment => todo!(),
                Token::Doctype => todo!(),
                Token::Tag { .. } if token.is_start_tag_with_name(&["html"]) => todo!(),
                Token::Tag { .. }
                    if token.is_start_tag_with_name(&[
                        "base", "basefont", "bgsound", "link", "meta", "noframes", "script",
                        "style", "template", "title",
                    ]) =>
                {
                    todo!()
                }
                Token::Tag { .. } if token.is_end_tag_with_name(&["template"]) => todo!(),
                Token::Tag { .. } if token.is_start_tag_with_name(&["body"]) => todo!(),
                Token::Tag { .. } if token.is_start_tag_with_name(&["frameset"]) => todo!(),
                Token::EndOfFile => {
                    // TODO: If the stack of template insertion modes is not empty, then process the
                    // token using the rules for the "in template" insertion
                    // mode.

                    // TODO: Otherwise, follow these steps:

                    // TODO: 1. If there is a node in the stack of open elements that is not either
                    // a dd element, a dt element, an li element, an optgroup element, an option
                    // element, a p element, an rb element, an rp element, an rt element, an rtc
                    // element, a tbody element, a td element, a tfoot element, a th element, a
                    // thead element, a tr element, the body element, or the html element, then this
                    // is a parse error.

                    self.stop_parsing();
                }
                Token::Tag { .. } if token.is_end_tag_with_name(&["body"]) => {
                    // TODO: If the stack of open elements does not have a body
                    // element in scope, this is a parse error; ignore the
                    // token.

                    // TODO: Otherwise, if there is a node in the stack of open
                    // elements that is not either a dd element, a dt element,
                    // an li element, an optgroup element, an option element, a
                    // p element, an rb element, an rp element, an rt element,
                    // an rtc element, a tbody element, a td element, a tfoot
                    // element, a th element, a thead element, a tr element, the
                    // body element, or the html element, then this is a parse
                    // error.

                    self.switch_insertion_mode(InsertionMode::AfterBody);
                }
                Token::Tag { .. } if token.is_end_tag_with_name(&["html"]) => todo!(),
                Token::Tag { .. }
                    if token.is_start_tag_with_name(&[
                        "address",
                        "article",
                        "aside",
                        "blockquote",
                        "center",
                        "details",
                        "dialog",
                        "dir",
                        "div",
                        "dl",
                        "fieldset",
                        "figcaption",
                        "figure",
                        "footer",
                        "header",
                        "hgroup",
                        "main",
                        "menu",
                        "nav",
                        "ol",
                        "p",
                        "search",
                        "section",
                        "summary",
                        "ul",
                    ]) =>
                {
                    // TODO: If the stack of open elements has a p element in
                    // button scope, then close a p element.

                    // TODO: Insert an HTML element for the token.
                }
                Token::Tag { .. }
                    if token.is_start_tag_with_name(&["h1", "h2", "h3", "h4", "h5", "h6"]) =>
                {
                    todo!()
                }
                Token::Tag { .. } if token.is_start_tag_with_name(&["pre", "listing"]) => todo!(),
                Token::Tag { .. } if token.is_start_tag_with_name(&["form"]) => todo!(),
                Token::Tag { .. } if token.is_start_tag_with_name(&["li"]) => todo!(),
                Token::Tag { .. } if token.is_start_tag_with_name(&["dd", "dt"]) => todo!(),
                Token::Tag { .. } if token.is_start_tag_with_name(&["plaintext"]) => todo!(),
                Token::Tag { .. } if token.is_start_tag_with_name(&["button"]) => todo!(),
                Token::Tag { .. }
                    if token.is_end_tag_with_name(&[
                        "address",
                        "article",
                        "aside",
                        "blockquote",
                        "button",
                        "center",
                        "details",
                        "dialog",
                        "dir",
                        "div",
                        "dl",
                        "fieldset",
                        "figcaption",
                        "figure",
                        "footer",
                        "header",
                        "hgroup",
                        "listing",
                        "main",
                        "menu",
                        "nav",
                        "ol",
                        "pre",
                        "search",
                        "section",
                        "summary",
                        "ul",
                    ]) =>
                {
                    todo!()
                }
                Token::Tag { .. } if token.is_end_tag_with_name(&["form"]) => todo!(),
                Token::Tag { .. } if token.is_end_tag_with_name(&["p"]) => {
                    // TODO: If the stack of open elements does not have a p
                    // element in button scope, then this is a parse error;
                    // insert an HTML element for a "p" start tag token with no
                    // attributes.

                    // TODO: Close a p element.
                }
                Token::Tag { .. } if token.is_end_tag_with_name(&["lo"]) => todo!(),
                Token::Tag { .. } if token.is_end_tag_with_name(&["dd", "dt"]) => todo!(),
                Token::Tag { .. }
                    if token.is_end_tag_with_name(&["h1", "h2", "h3", "h4", "h5", "h6"]) =>
                {
                    todo!()
                }
                Token::Tag { .. } if token.is_start_tag_with_name(&["a"]) => todo!(),
                Token::Tag { .. }
                    if token.is_start_tag_with_name(&[
                        "b", "big", "code", "em", "font", "i", "s", "small", "strike", "strong",
                        "tt", "u",
                    ]) =>
                {
                    todo!()
                }
                Token::Tag { .. } if token.is_start_tag_with_name(&["nobr"]) => todo!(),
                Token::Tag { .. }
                    if token.is_end_tag_with_name(&[
                        "a", "b", "big", "code", "em", "font", "i", "nobr", "s", "small", "strike",
                        "strong", "tt", "u",
                    ]) =>
                {
                    todo!()
                }
                Token::Tag { .. }
                    if token.is_start_tag_with_name(&["applet", "marquee", "object"]) =>
                {
                    todo!()
                }
                Token::Tag { .. }
                    if token.is_end_tag_with_name(&["applet", "marquee", "object"]) =>
                {
                    todo!()
                }
                Token::Tag { .. } if token.is_start_tag_with_name(&["table"]) => todo!(),
                Token::Tag { .. } if token.is_end_tag_with_name(&["br"]) => todo!(),
                Token::Tag { .. }
                    if token.is_start_tag_with_name(&[
                        "area", "br", "embed", "img", "keygen", "wbr",
                    ]) =>
                {
                    todo!()
                }
                Token::Tag { .. } if token.is_start_tag_with_name(&["input"]) => todo!(),
                Token::Tag { .. }
                    if token.is_start_tag_with_name(&["param", "source", "track"]) =>
                {
                    todo!()
                }
                Token::Tag { .. } if token.is_start_tag_with_name(&["hr"]) => todo!(),
                Token::Tag { .. } if token.is_start_tag_with_name(&["image"]) => todo!(),
                Token::Tag { .. } if token.is_start_tag_with_name(&["textarea"]) => todo!(),
                Token::Tag { .. } if token.is_start_tag_with_name(&["xmp"]) => todo!(),
                Token::Tag { .. } if token.is_start_tag_with_name(&["iframe"]) => todo!(),
                Token::Tag { .. } if token.is_start_tag_with_name(&["noembed"]) => todo!(),
                Token::Tag { .. }
                    if token.is_start_tag_with_name(&["noscript"]) && self.scripting =>
                {
                    todo!()
                }
                Token::Tag { .. } if token.is_start_tag_with_name(&["select"]) => todo!(),
                Token::Tag { .. } if token.is_start_tag_with_name(&["optgroup", "option"]) => {
                    todo!()
                }
                Token::Tag { .. } if token.is_start_tag_with_name(&["rb", "rtc"]) => todo!(),
                Token::Tag { .. } if token.is_start_tag_with_name(&["rp", "rt"]) => todo!(),
                Token::Tag { .. } if token.is_start_tag_with_name(&["math"]) => todo!(),
                Token::Tag { .. } if token.is_start_tag_with_name(&["svg"]) => todo!(),
                Token::Tag { .. }
                    if token.is_start_tag_with_name(&[
                        "caption", "col", "colgroup", "frame", "head", "tbody", "td", "tfoot",
                        "th", "thead", "tr",
                    ]) =>
                {
                    todo!()
                }
                Token::Tag { .. } if token.is_start_tag() => todo!(),
                Token::Tag { .. } if token.is_end_tag() => todo!(),
                _ => unreachable!(),
            },
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
            InsertionMode::AfterBody => match token {
                whitespace!() => self.process_token(InsertionMode::InBody, token),
                Token::Comment => todo!(),
                Token::Doctype => todo!(),
                Token::Tag { .. } if token.is_start_tag_with_name(&["html"]) => {
                    self.process_token(InsertionMode::InBody, token);
                }
                Token::Tag { .. } if token.is_end_tag_with_name(&["html"]) => {
                    self.process_token(InsertionMode::AfterAfterBody, token);
                }
                Token::EndOfFile => self.stop_parsing(),
                _ => todo!(),
            },
            InsertionMode::InFrameset => todo!("InFrameset"),
            InsertionMode::AfterFrameset => todo!("AfterFrameset"),
            InsertionMode::AfterAfterBody => match token {
                Token::Comment => todo!(),
                Token::Doctype => todo!(),
                whitespace!() | Token::Tag { .. } if token.is_start_tag_with_name(&["html"]) => {
                    self.process_token(InsertionMode::InBody, token);
                }
                Token::EndOfFile => self.stop_parsing(),
                _ => {
                    // TODO: Parsing error.

                    self.switch_insertion_mode(InsertionMode::InBody);
                }
            },
            InsertionMode::AfterAfterFrameset => todo!("AfterAfterFrameset"),
        }
    }

    fn stop_parsing(&mut self) {
        self.should_stop_parsing = true;
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
