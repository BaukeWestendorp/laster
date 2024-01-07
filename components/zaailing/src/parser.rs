use crate::arena::{NodeArena, NodeId};
use crate::node::{Node, NodeKind};
use crate::tokenizer::{self, Token};

pub enum Namespace {
    Html,
}

impl Namespace {
    pub fn url(&self) -> &str {
        match self {
            Namespace::Html => "http://www.w3.org/1999/xhtml",
        }
    }
}

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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct InsertionLocation {
    parent: NodeId,
    after_child: Option<NodeId>,
}

impl InsertionLocation {
    /// https://html.spec.whatwg.org/multipage/parsing.html#insert-an-element-at-the-adjusted-insertion-location
    pub fn insert_element(&self, arena: &mut NodeArena, element: NodeId) {
        // TODO: If it is not possible to insert element at the adjusted
        // insertion location, abort these steps.

        // TODO: If the parser was not created as part of the HTML fragment
        // parsing algorithm, then push a new element queue onto
        // element's relevant agent's custom element reactions stack.

        // Insert element at the adjusted insertion location.
        arena.insert(element, self.parent, self.after_child)

        // TODO: If the parser was not created as part of the HTML fragment
        // parsing algorithm, then pop the element queue from element's
        // relevant agent's custom element reactions stack, and invoke
        // custom element reactions in that queue.
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParsingAlgorithm {
    RawText,
    RcData,
}

#[derive(Debug)]
pub struct Parser<'input, 'arena> {
    arena: &'arena mut NodeArena,
    tokenizer: tokenizer::Tokenizer<'input>,
    insertion_mode: InsertionMode,
    original_insertion_mode: InsertionMode,
    should_reprocess_token: bool,
    document: NodeId,
    stack_of_open_elements: StackOfOpenElements,
    active_formatting_elements: ActiveFormattingElements,
    head_element: Option<NodeId>,
    should_stop_parsing: bool,
    scripting: bool,
    frameset_ok: bool,
    foster_parenting: bool,
}

impl<'input, 'arena> Parser<'input, 'arena> {
    pub fn new(html: &'input str, arena: &'arena mut NodeArena) -> Self {
        Self {
            tokenizer: tokenizer::Tokenizer::new(html),
            insertion_mode: InsertionMode::Initial,
            original_insertion_mode: InsertionMode::Initial,
            should_reprocess_token: false,
            document: arena.create_node(Node::create_document()),
            stack_of_open_elements: StackOfOpenElements::new(),
            active_formatting_elements: ActiveFormattingElements::new(),
            head_element: None,
            should_stop_parsing: false,
            scripting: false,
            frameset_ok: true,
            foster_parenting: false,
            arena,
        }
    }

    pub fn parse(mut self) -> Node {
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

        self.arena.get_node(self.document).clone()
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
                Token::Comment { .. } => {
                    todo!("Insert a comment as the last child of the Document object.");
                }
                Token::Doctype {
                    name,
                    public_identifier,
                    system_identifier,
                } => {
                    // If the DOCTYPE token's name is not "html", or the token's
                    // public identifier is not missing, or the token's system
                    // identifier is neither missing nor "about:legacy-compat",
                    // then there is a parse error.
                    if name != "html"
                        || public_identifier.is_some()
                        || system_identifier.is_some()
                            && system_identifier != &Some("about:legacy-compat".to_string())
                    {
                        self.error("Invalid DOCTYPE");
                    }

                    // Append a DocumentType node to the Document node, with its
                    // name set to the name given in the DOCTYPE token, or the
                    // empty string if the name was missing; its public ID set
                    // to the public identifier given in the DOCTYPE token, or
                    // the empty string if the public identifier was missing;
                    // and its system ID set to the system identifier given in
                    // the DOCTYPE token, or the empty string if the system
                    // identifier was missing.
                    let doctype = Node::create_doctype(
                        self.document,
                        name.clone(),
                        public_identifier.clone().unwrap_or_default(),
                        system_identifier.clone().unwrap_or_default(),
                    );
                    let doctype = self.arena.create_node(doctype);
                    self.arena.append(doctype, self.document);

                    // TODO: Then, if the document is not an iframe srcdoc
                    // document, and the parser cannot
                    // change the mode flag is false, and
                    // the DOCTYPE token matches one of the conditions in the
                    // following list, then set the Document to quirks mode:

                    // TODO: Otherwise, if the document is not an iframe srcdoc
                    // document, and the parser cannot change the mode flag is
                    // false, and the DOCTYPE token matches one of the
                    // conditions in the following list, then then set the
                    // Document to limited-quirks mode:
                    //
                    // The system identifier and public identifier strings must
                    // be compared to the values given in the lists above in an
                    // ASCII case-insensitive manner. A system identifier whose
                    // value is the empty string is not considered missing for
                    // the purposes of the conditions above.

                    // Then, switch the insertion mode to "before html".
                    self.switch_insertion_mode(InsertionMode::BeforeHtml);
                }
                _ => {
                    // TODO: If the document is not an iframe srcdoc document, then this is a parse
                    // error; if the parser cannot change the mode flag is false, set the Document
                    // to quirks mode.

                    self.switch_insertion_mode_and_reprocess_token(InsertionMode::BeforeHtml);
                }
            },
            InsertionMode::BeforeHtml => {
                match token {
                    Token::Doctype { .. } => {
                        self.error("Unexpected DOCTYPE");
                    }
                    Token::Comment { .. } => {
                        todo!("Insert a comment as the last child of the Document object.");
                    }
                    whitespace!() => {}
                    Token::Tag { .. } if token.is_start_tag_with_name(&["html"]) => {
                        let html_element =
                            self.create_element_for_token(token, Namespace::Html, self.document);
                        self.arena.append(html_element, self.document);
                        self.stack_of_open_elements.push(html_element);
                        self.switch_insertion_mode(InsertionMode::BeforeHead);
                    }
                    Token::Tag { .. }
                        if token.is_end_tag_with_name(&["head", "body", "html", "br"]) =>
                    {
                        todo!("Act as described in the 'anything else' entry below.");
                    }
                    Token::Tag { .. } if token.is_end_tag() => {
                        self.error("Unexpected end tag");
                    }
                    _ => {
                        // TODO: Create an html element whose node document is the Document object.
                        // Append it to the Document object. Put this element in the stack of open
                        // elements.

                        self.switch_insertion_mode_and_reprocess_token(InsertionMode::BeforeHead);
                    }
                }
            }
            InsertionMode::BeforeHead => match token {
                whitespace!() => {}
                Token::Comment { .. } => {
                    todo!("Insert a comment.");
                }
                Token::Doctype { .. } => {
                    self.error("Unexpected DOCTYPE");
                }
                Token::Tag { .. } if token.is_start_tag_with_name(&["html"]) => {
                    self.process_token(InsertionMode::InBody, token);
                }
                Token::Tag { .. } if token.is_start_tag_with_name(&["head"]) => {
                    let head = self.insert_html_element(&token);
                    self.head_element = Some(head);
                    self.switch_insertion_mode(InsertionMode::InHead);
                }
                Token::Tag { .. }
                    if token.is_end_tag_with_name(&["head", "body", "html", "br"]) =>
                {
                    todo!("Act as described in the 'anything else' entry below.");
                }
                Token::Tag { .. } if token.is_end_tag() => {
                    self.error("Unexpected end tag");
                }
                _ => {
                    todo!();
                }
            },
            InsertionMode::InHead => match token {
                whitespace!() => {
                    // Insert the character.
                    let character = match token {
                        Token::Character(character) => character,
                        _ => unreachable!(),
                    };

                    self.insert_character(*character);
                }
                Token::Comment { .. } => {
                    todo!("Insert a comment.");
                }
                Token::Doctype { .. } => {
                    self.error("Unexpected DOCTYPE");
                }
                Token::Tag { .. } if token.is_start_tag_with_name(&["html"]) => {
                    self.process_token(InsertionMode::InBody, token);
                }
                Token::Tag { .. }
                    if token.is_start_tag_with_name(&["base", "basefont", "bgsound", "link"]) =>
                {
                    // Insert an HTML element for the token.
                    self.insert_html_element(token);

                    // Immediately pop the current node off the stack of open elements.
                    self.stack_of_open_elements.pop();

                    // Acknowledge the token's self-closing flag, if it is set.
                    token.acknowledge_self_closing_flag();
                }
                Token::Tag { .. } if token.is_start_tag_with_name(&["meta"]) => {
                    // Insert an HTML element for the token. Immediately pop the
                    // current node off the stack of open elements.
                    self.insert_html_element(token);
                    self.stack_of_open_elements.pop();

                    // Acknowledge the token's self-closing flag, if it is
                    // set.
                    token.acknowledge_self_closing_flag();

                    // TODO: If the active speculative HTML parser is null,
                    // then:
                }
                Token::Tag { .. } if token.is_start_tag_with_name(&["title"]) => {
                    // Follow the generic RCDATA element parsing algorithm.
                    self.follow_generic_parsing_algorithm(token, ParsingAlgorithm::RcData);
                }
                Token::Tag { .. }
                    if (token.is_start_tag_with_name(&["noscript"]) && self.scripting)
                        || token.is_start_tag_with_name(&["noframes", "style"]) =>
                {
                    // Follow the generic raw text element parsing algorithm.
                    self.follow_generic_parsing_algorithm(token, ParsingAlgorithm::RawText);
                }
                Token::Tag { .. } if token.is_start_tag_with_name(&["script"]) => {
                    todo!();
                }
                Token::Tag { .. } if token.is_end_tag_with_name(&["head"]) => {
                    // Pop the current node (which will be the head element) off the stack of
                    // open elements.
                    self.stack_of_open_elements.pop();

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
                    self.error("Unexpected tag");
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
                    // Insert the character.
                    let character = match token {
                        Token::Character(character) => character,
                        _ => unreachable!(),
                    };
                    self.insert_character(*character);
                }
                Token::Comment { .. } => {
                    todo!("Insert a comment.");
                }
                Token::Doctype { .. } => {
                    self.error("Unexpected DOCTYPE");
                }
                Token::Tag { .. } if token.is_start_tag_with_name(&["html"]) => {
                    self.process_token(InsertionMode::InBody, token)
                }
                Token::Tag { .. } if token.is_start_tag_with_name(&["body"]) => {
                    self.insert_html_element(token);
                    self.frameset_ok = false;
                    self.switch_insertion_mode(InsertionMode::InBody);
                }
                Token::Tag { .. } if token.is_start_tag_with_name(&["frameset"]) => {
                    self.insert_html_element(token);
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
                    self.error("Unexpected tag");
                }
                _ => {
                    self.insert_html_element(&Token::Tag {
                        start: true,
                        tag_name: "body".to_string(),
                        attributes: vec![],
                        self_closing: false,
                    });
                    self.switch_insertion_mode_and_reprocess_token(InsertionMode::InBody);
                }
            },
            InsertionMode::InBody => match token {
                Token::Character('\0') => {
                    // Parse error. Ignore the token.
                    self.error("Unexpected null character");
                }
                whitespace!() => {
                    // Reconstruct the active formatting elements, if any.
                    self.active_formatting_elements
                        .reconstruct(&self.stack_of_open_elements);

                    let character = match token {
                        Token::Character(character) => character,
                        _ => unreachable!(),
                    };

                    // Insert the token's character.
                    self.insert_character(*character);
                }
                Token::Character(character) => {
                    // Reconstruct the active formatting elements, if any.
                    self.active_formatting_elements
                        .reconstruct(&self.stack_of_open_elements);

                    // Insert the token's character.
                    self.insert_character(*character);

                    // Set the frameset-ok flag to "not ok".
                    self.frameset_ok = false;
                }
                Token::Comment { .. } => todo!(),
                Token::Doctype { .. } => {
                    // Parse error. Ignore the token.
                    self.error("Unexpected DOCTYPE");
                }
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
                    // If the stack of open elements has a p element in
                    // button scope, then close a p element.
                    if self
                        .stack_of_open_elements
                        .has_element_in_button_scope(&self.arena, "p")
                    {
                        self.close_p_element();
                    }

                    // Insert an HTML element for the token.
                    self.insert_html_element(token);
                }
                Token::Tag { .. }
                    if token.is_start_tag_with_name(&["h1", "h2", "h3", "h4", "h5", "h6"]) =>
                {
                    // If the stack of open elements has a p element in button
                    // scope, then close a p element.
                    if self
                        .stack_of_open_elements
                        .has_element_in_button_scope(&self.arena, "p")
                    {
                        self.close_p_element();
                    }

                    // If the current node is an HTML element whose tag name is
                    // one of "h1", "h2", "h3", "h4", "h5", or "h6", then this
                    // is a parse error; pop the current node off the stack of
                    // open elements.
                    if self
                        .arena
                        .get_node(self.stack_of_open_elements.current_node())
                        .is_element_with_one_of_tag_names(&["h1", "h2", "h3", "h4", "h5", "h6"])
                    {
                        self.error("Unexpected tag");
                        self.stack_of_open_elements.pop();
                    }

                    // Insert an HTML element for the token.
                    self.insert_html_element(token);
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
                    // If the stack of open elements does not have a p element in button scope,
                    if !self
                        .stack_of_open_elements
                        .has_element_in_button_scope(&self.arena, "p")
                    {
                        // then this is a parse error;
                        self.error("Expected p element in button scope");

                        // insert an HTML element for a "p" start tag token with no attributes.
                        self.insert_html_element(&Token::Tag {
                            start: true,
                            tag_name: "p".to_string(),
                            attributes: vec![],
                            self_closing: false,
                        });
                    }

                    // Close a p element.
                    self.close_p_element();
                }
                Token::Tag { .. } if token.is_end_tag_with_name(&["lo"]) => todo!(),
                Token::Tag { .. } if token.is_end_tag_with_name(&["dd", "dt"]) => todo!(),
                Token::Tag { .. }
                    if token.is_end_tag_with_name(&["h1", "h2", "h3", "h4", "h5", "h6"]) =>
                {
                    todo!()
                }
                Token::Tag { .. } if token.is_start_tag_with_name(&["a"]) => {
                    // TODO: If the list of active formatting elements contains
                    // an a element between the end of the
                    // list and the last marker on the list
                    // (or the start of the list if there is no
                    // marker on the list), then this is a parse error; run the
                    // adoption agency algorithm for the token, then remove that
                    // element from the list of active formatting elements and
                    // the stack of open elements if the adoption agency
                    // algorithm didn't already remove it (it might not have if
                    // the element is not in table scope).

                    // Reconstruct the active formatting elements, if any.
                    self.active_formatting_elements
                        .reconstruct(&self.stack_of_open_elements);

                    // Insert an HTML element for the token. Push onto the list
                    // of active formatting elements that element.
                    let element = self.insert_html_element(token);
                    self.active_formatting_elements
                        .push(ActiveFormattingElement::Element(element));
                }
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
                    // Run the adoption agency algorithm for the token.
                    self.run_adoption_agency_algorithm(token);
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
            InsertionMode::Text => {
                match token {
                    Token::Character(char) => {
                        // Insert the token's character.
                        self.insert_character(*char);
                    }
                    Token::EndOfFile => {
                        // Parse error.
                        self.error("Unexpected end of file");

                        // TODO: If the current node is a script element, then
                        // set its already started to
                        // true.

                        // Pop the current node off the stack of open elements.
                        self.stack_of_open_elements.pop();

                        // Switch the insertion mode to the original insertion
                        // mode and reprocess the token.
                        self.switch_insertion_mode_and_reprocess_token(
                            self.original_insertion_mode,
                        );
                    }
                    Token::Tag { .. } if token.is_end_tag_with_name(&["script"]) => {
                        todo!();
                    }
                    _ => {
                        // Pop the current node off the stack of open elements.
                        self.stack_of_open_elements.pop();

                        // Switch the insertion mode to the original insertion
                        // mode.
                        self.switch_insertion_mode(self.original_insertion_mode);
                    }
                }
            }
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
                Token::Comment { .. } => todo!(),
                Token::Doctype { .. } => todo!(),
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
                Token::Comment { .. } => todo!(),
                Token::Doctype { .. } => todo!(),
                whitespace!() | Token::Tag { .. } if token.is_start_tag_with_name(&["html"]) => {
                    self.process_token(InsertionMode::InBody, token);
                }
                Token::EndOfFile => self.stop_parsing(),
                _ => {
                    self.error(format!("Unexpected token: {:?}", token).as_str());

                    self.switch_insertion_mode(InsertionMode::InBody);
                }
            },
            InsertionMode::AfterAfterFrameset => todo!("AfterAfterFrameset"),
        }
    }

    /// https://html.spec.whatwg.org/multipage/parsing.html#adoption-agency-algorithm
    fn run_adoption_agency_algorithm(&mut self, token: &Token) {
        // 1. Let subject be token's tag name.
        let subject = match token {
            Token::Tag { tag_name, .. } => tag_name,
            _ => panic!("Expected tag token"),
        };

        // If the current node is an HTML element whose tag name is subject, and the
        // current node is not in the list of active formatting elements, then pop the
        // current node off the stack of open elements and return.
        let current_node = self.stack_of_open_elements.current_node();
        if self
            .arena
            .get_node(current_node)
            .is_element_with_tag_name(&subject)
            && !self.active_formatting_elements.contains(current_node)
        {
            self.stack_of_open_elements.pop();
            return;
        }

        // Let outer loop counter be 0.
        let mut outer_loop_counter = 0;

        // While true:
        loop {
            // If outer loop counter is greater than or equal to 8, then return.
            if outer_loop_counter >= 8 {
                return;
            }

            // Increment outer loop counter by 1.
            outer_loop_counter += 1;

            // Let formatting element be the last element in the list of active
            // formatting elements that:
            //     * is between the end of the list and the last marker in the list, if any,
            //       or the start of the list otherwise, and
            //     * has the tag name subject.
            let formatting_element = self
                .active_formatting_elements
                .last_element_with_tag_name_before_marker(&self.arena, &subject);

            // If there is no such element, then return and instead act as described in the
            // "any other end tag" entry above.
            if formatting_element.is_none() {
                todo!();
            }
            let formatting_element = formatting_element.unwrap();

            // If formatting element is not in the stack of open elements,
            if !self.stack_of_open_elements.contains(formatting_element) {
                // then this is a parse error;
                self.error("Formatting element not in the stack of open elements");
                // remove the element from the list
                self.active_formatting_elements.remove(formatting_element);
                // and return.
                return;
            }

            // If formatting element is in the stack of open elements, but the element
            // is not in scope,
            let formatting_element_tag_name = match &self.arena.get_node(formatting_element).kind {
                NodeKind::Element { tag_name, .. } => tag_name,
                _ => panic!("Formatting element is not an element"),
            };
            if !self
                .stack_of_open_elements
                .has_element_in_scope(&self.arena, &formatting_element_tag_name)
            {
                // then this is a parse error; return.
                self.error("Formatting element is not in scope");
                return;
            }

            // If formatting element is not the current node,
            if formatting_element != self.stack_of_open_elements.current_node() {
                // this is a parse error. (But do not return.)
                self.error("Formatting element is not the current node");
            }

            // Let furthest block be the topmost node in the stack of open elements that
            // is lower in the stack than formatting element, and is an element in the
            // special category. There might not be one.
            let furthest_block = self
                .stack_of_open_elements
                .topmost_special_node_below(&self.arena, formatting_element);

            // If there is no furthest block, then the UA must first pop all the nodes
            // from the bottom of the stack of open elements, from the current node up to
            // and including formatting element,
            if furthest_block.is_none() {
                while formatting_element != self.stack_of_open_elements.current_node() {
                    self.stack_of_open_elements.pop();
                }
                self.stack_of_open_elements.pop();

                // then remove formatting element from the list of active formatting elements,
                self.active_formatting_elements.remove(formatting_element);
                // and finally return.
                return;
            }
            let furthest_block = furthest_block.unwrap();

            // Let common ancestor be the element immediately above formatting element
            // in the stack of open elements.
            let common_ancestor = self
                .stack_of_open_elements
                .element_immediately_above(formatting_element);

            // Let a bookmark note the position of formatting element in the list of
            // active formatting elements relative to the elements on either side of it in
            // the list.
            let mut bookmark = self
                .active_formatting_elements
                .first_index_of(formatting_element)
                .unwrap();

            // Let node and last node be furthest block.
            let mut node = furthest_block;
            let mut last_node = furthest_block;

            let node_above_node = self.stack_of_open_elements.element_immediately_above(node);

            // Let inner loop counter be 0.
            let mut inner_loop_count = 0;

            // While true:
            loop {
                // Increment inner loop counter by 1.
                inner_loop_count += 1;

                // Let node be the element immediately above node in the stack of open
                // elements, or if node is no longer in the stack of open elements (e.g. because
                // it got removed by this algorithm), the element that was immediately above
                // node in the stack of open elements before node was removed.
                if let Some(node_above_node) = node_above_node {
                    node = node_above_node;
                }

                // If node is formatting element, then break.
                if node == formatting_element {
                    break;
                }

                // If inner loop counter is greater than 3 and node is in the list of
                // active formatting elements, then remove node from the list of active
                // formatting elements.
                if inner_loop_count > 3 && self.active_formatting_elements.contains(node) {
                    self.active_formatting_elements.remove(node);
                }

                // If node is not in the list of active formatting elements, then remove
                // node from the stack of open elements and continue.
                if !self.active_formatting_elements.contains(node) {
                    self.stack_of_open_elements.remove_element(node);
                    continue;
                }

                // Create an element for the token for which the element node was
                // created, in the HTML namespace, with common ancestor as the intended parent;
                let new_element =
                    self.create_element_for_token(token, Namespace::Html, common_ancestor.unwrap());

                // replace the entry for node in the list of active
                // formatting elements with an entry for the new element,
                self.active_formatting_elements.replace(node, new_element);

                // replace the entry for node in the stack of open elements
                // with an entry for the new element,
                self.stack_of_open_elements.replace(node, new_element);

                // and let node be the new element.
                node = new_element;

                // If last node is furthest block, then move the aforementioned bookmark
                // to be immediately after the new node in the list of active formatting
                // elements.
                if last_node == furthest_block {
                    bookmark = self
                        .active_formatting_elements
                        .first_index_of(node)
                        .unwrap()
                        + 1
                }

                // Append last node to node.
                self.arena.append(last_node, node);

                // Set last node to node.
                last_node = node;
            }

            // Insert whatever last node ended up being in the previous step at the
            // appropriate place for inserting a node, but using common ancestor as the
            // override target.
            let adjusted_insertion_location =
                self.appropriate_place_for_inserting_node(common_ancestor);
            self.arena.insert(
                last_node,
                adjusted_insertion_location.parent,
                adjusted_insertion_location.after_child,
            );

            // Create an element for the token for which formatting element was created,
            // in the HTML namespace, with furthest block as the intended parent.
            let new_element = self.create_element_for_token(token, Namespace::Html, furthest_block);

            // Take all of the child nodes of furthest block and append them to the
            // element created in the last step.
            let children = self.arena.get_node(furthest_block).children().to_vec();
            for child in children.iter() {
                self.arena.append(*child, new_element);
            }

            // Append that new element to furthest block.
            self.arena.append(new_element, furthest_block);

            // Remove formatting element from the list of active formatting elements,
            self.active_formatting_elements.remove(formatting_element);
            // and insert the new element into the list of active formatting elements at the
            // position of the aforementioned bookmark.
            self.active_formatting_elements
                .insert(bookmark, new_element);

            // Remove formatting element from the stack of open elements,
            self.stack_of_open_elements
                .remove_element(formatting_element);

            // and insert the new element into the stack of open elements immediately below
            // the position of furthest block in that stack.
            self.stack_of_open_elements
                .insert_immediately_below(new_element, furthest_block);
        }
    }

    /// https://html.spec.whatwg.org/multipage/parsing.html#parsing-elements-that-contain-only-text
    fn follow_generic_parsing_algorithm(&mut self, token: &Token, algorithm: ParsingAlgorithm) {
        // Insert an HTML element for the token.
        self.insert_html_element(token);

        // If the algorithm that was invoked is the generic raw text element
        // parsing algorithm, switch the tokenizer to the RAWTEXT state;
        // otherwise the algorithm invoked was the generic RCDATA element
        // parsing algorithm, switch the tokenizer to the RCDATA state.
        match algorithm {
            ParsingAlgorithm::RawText => self.tokenizer.switch_to(tokenizer::State::RawText),
            ParsingAlgorithm::RcData => self.tokenizer.switch_to(tokenizer::State::RcData),
        }

        // Let the original insertion mode be the current insertion mode.
        self.original_insertion_mode = self.insertion_mode;

        // Then, switch the insertion mode to "text".
        self.switch_insertion_mode(InsertionMode::Text);
    }

    /// https://html.spec.whatwg.org/multipage/parsing.html#insert-a-character
    fn insert_character(&mut self, data: char) {
        // Let the adjusted insertion location be the appropriate place for
        // inserting a node.
        let adjusted_insertion_location = self.appropriate_place_for_inserting_node(None);

        // If the adjusted insertion location is in a Document node, then
        // return.
        if self
            .arena
            .get_node(adjusted_insertion_location.parent)
            .is_document()
        {
            return;
        }

        // If there is a Text node immediately before the adjusted insertion
        // location, then append data to that Text node's data.
        match adjusted_insertion_location.after_child {
            Some(after) => {
                if let Some(previous_sibling) = self.arena.previous_sibling(after) {
                    if let NodeKind::Text { data: text } =
                        &mut self.arena.get_node_mut(previous_sibling).kind
                    {
                        text.push(data);
                        return;
                    }
                }
            }
            None => {
                if let Some(last_child) = self
                    .arena
                    .get_node(adjusted_insertion_location.parent)
                    .children()
                    .last()
                {
                    if let NodeKind::Text { data: text } =
                        &mut self.arena.get_node_mut(*last_child).kind
                    {
                        text.push(data);
                        return;
                    }
                }
            }
        };

        // Otherwise, create a new Text node whose data is data and whose node
        // document is the same as that of the element in which the adjusted
        // insertion location finds itself, and insert the newly created node at
        // the adjusted insertion location.
        let document = self
            .arena
            .get_node(adjusted_insertion_location.parent)
            .node_document(self.arena);

        let text_node = Node::create_text(document, data.to_string());
        let text_node_id = self.arena.create_node(text_node);
        adjusted_insertion_location.insert_element(self.arena, text_node_id);
    }

    /// https://html.spec.whatwg.org/multipage/parsing.html#insert-a-foreign-element
    fn insert_foreign_element(
        &mut self,
        token: &Token,
        namespace: Namespace,
        only_add_to_element_stack: bool,
    ) -> NodeId {
        // Let the adjusted insertion location be the appropriate place for
        // inserting a node.
        let adjusted_insertion_location = self.appropriate_place_for_inserting_node(None);

        // Let element be the result of creating an element for the token in the
        // given namespace, with the intended parent being the element in which
        // the adjusted insertion location finds itself.
        let element =
            self.create_element_for_token(&token, namespace, adjusted_insertion_location.parent);

        // If onlyAddToElementStack is false, then run insert an element at the
        // adjusted insertion location with element.
        if !only_add_to_element_stack {
            adjusted_insertion_location.insert_element(&mut self.arena, element);
        }

        // Push element onto the stack of open elements so that it is the new
        // current node.
        self.stack_of_open_elements.push(element);

        // Return element.
        element
    }

    /// https://html.spec.whatwg.org/multipage/parsing.html#insert-an-html-element
    fn insert_html_element(&mut self, token: &Token) -> NodeId {
        self.insert_foreign_element(token, Namespace::Html, false)
    }

    /// https://html.spec.whatwg.org/multipage/parsing.html#create-an-element-for-the-token
    fn create_element_for_token(
        &mut self,
        token: &Token,
        namespace: Namespace,
        intended_parent: NodeId,
    ) -> NodeId {
        // TODO: If the active speculative HTML parser is not null, then return
        // the result of creating a speculative mock element given given
        // namespace, the tag name of the given token, and the
        // attributes of the given token.

        // TODO: Otherwise, optionally create a speculative mock element given
        // given namespace, the tag name of the given token, and the
        // attributes of the given token.

        // Let document be intended parent's node document.
        let document = self
            .arena
            .get_node(intended_parent)
            .node_document(&self.arena);

        // Let local name be the tag name of the token.
        let local_name = match token {
            Token::Tag { tag_name, .. } => tag_name,
            _ => panic!("Expected Token::Tag token, got {:?}", token),
        };

        // TODO: Let is be the value of the "is" attribute in the given token,
        // if such an attribute exists, or null otherwise.
        let is = None;

        // TODO: Let definition be the result of looking up a custom element
        // definition given document, given namespace, local name, and is.

        // TODO: If definition is non-null and the parser was not created as
        // part of the HTML fragment parsing algorithm, then let will execute
        // script be true. Otherwise, let it be false.
        let execute_script = false;

        // If will execute script is true, then:
        if execute_script {
            // TODO: (See spec)
        }

        // Let element be the result of creating an element given
        // document, localName, given namespace, null, and is. If will execute
        // script is true, set the synchronous custom elements flag; otherwise,
        // leave it unset.
        let element = Node::create_element(
            document,
            local_name.clone(),
            namespace,
            None,
            is,
            execute_script,
        );

        // TODO: Append each attribute in the given token to element.

        // If will execute script is true, then:
        if execute_script {
            // TODO: (See spec)
        }

        // TODO: If element has an xmlns attribute in the XMLNS namespace whose
        // value is not exactly the same as the element's namespace, that is a
        // parse error. Similarly, if element has an xmlns:xlink attribute in
        // the XMLNS namespace whose value is not the XLink Namespace, that is a
        // parse error.

        // TODO: If element is a resettable element, invoke its reset algorithm.
        // (This initializes the element's value and checkedness based on the
        // element's attributes.)

        // TODO: If element is a form-associated element and not a
        // form-associated custom element, the form element pointer is not null,
        // there is no template element on the stack of open elements, element
        // is either not listed or doesn't have a form attribute, and the
        // intended parent is in the same tree as the element pointed to by the
        // form element pointer, then associate element with the form element
        // pointed to by the form element pointer and set element's parser
        // inserted flag.

        // Return element.
        self.arena.create_node(element)
    }

    /// https://html.spec.whatwg.org/multipage/parsing.html#appropriate-place-for-inserting-a-node
    fn appropriate_place_for_inserting_node(
        &self,
        override_target: Option<NodeId>,
    ) -> InsertionLocation {
        let target = match override_target {
            // If there was an override target specified, then let target be the override target.
            Some(override_target) => override_target,
            // Otherwise, let target be the current node.
            None => self.stack_of_open_elements.current_node(),
        };

        // Determine the adjusted insertion location using the first matching
        // steps from the following list:
        let adjusted_insertion_location = if self.foster_parenting {
            todo!("Implement foster parenting")
        } else {
            // Let adjusted insertion location be inside target, after its last child (if
            // any).
            InsertionLocation {
                parent: target,
                after_child: None,
            }
        };

        // TODO: If the adjusted insertion location is inside a template
        // element, let it instead be inside the template element's template
        // contents, after its last child (if any).

        // Return the adjusted insertion location.
        adjusted_insertion_location
    }

    /// https://html.spec.whatwg.org/multipage/parsing.html#close-a-p-element
    fn close_p_element(&mut self) {
        // Generate implied end tags, except for p elements.
        self.generate_implied_end_tags_except_for("p");

        // If the current node is not a p element, then this is a parse error.
        if !self
            .arena
            .get_node(self.stack_of_open_elements.current_node())
            .is_element_with_tag_name("p")
        {
            self.error("Expected current node to be a p element while closing a p element");
        }

        // Pop elements from the stack of open elements until a p element has been
        // popped from the stack.
        self.stack_of_open_elements
            .pop_until_element_with_tag_name(&self.arena, "p");
    }

    /// https://html.spec.whatwg.org/multipage/parsing.html#generate-implied-end-tags
    fn generate_implied_end_tags_except_for(&mut self, except: &str) {
        // while the current node is a dd element, a dt element, an li element, an
        // optgroup element, an option element, a p element, an rb element, an rp
        // element, an rt element, or an rtc element, the UA must pop the current node
        // off the stack of open elements.
        loop {
            let node = self
                .arena
                .get_node(self.stack_of_open_elements.current_node());

            if node.is_element_with_tag_name(except) {
                return;
            }

            if !node.is_element_with_one_of_tag_names(&[
                "dd", "dt", "li", "optgroup", "option", "p", "rb", "rp", "rt", "rtc",
            ]) {
                return;
            }

            self.stack_of_open_elements.pop();
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

    fn is_in_foreign_content(&self, token: &Token) -> bool {
        // If the stack of open elements is empty
        if self.stack_of_open_elements.is_empty() {
            return false;
        }

        let acn = self
            .arena
            .get_node(self.stack_of_open_elements.adjusted_current_node());

        // If the adjusted current node is an element in the HTML namespace
        if acn.is_element_in_namespace(Namespace::Html) {
            return false;
        }

        // TODO: If the adjusted current node is a MathML text integration point and the
        // token is a start tag whose tag name is neither "mglyph" nor
        // "malignmark"

        // TODO: If the adjusted current node is a MathML text integration point and the
        // token is a character token

        // TODO: If the adjusted current node is a MathML annotation-xml element and the
        // token is a start tag whose tag name is "svg"

        // TODO: If the adjusted current node is an HTML integration point and the token
        // is a start tag

        // TODO: If the adjusted current node is an HTML integration point and the token
        // is a character token

        // If the token is an end-of-file token
        if token == &Token::EndOfFile {
            return false;
        }

        true
    }

    fn error(&mut self, message: &str) {
        eprintln!("Parser error: {}", message);
    }
}

pub static SPECIAL_TAGS: &[&str] = &[
    "address",
    "applet",
    "area",
    "article",
    "aside",
    "base",
    "basefont",
    "bgsound",
    "blockquote",
    "body",
    "br",
    "button",
    "caption",
    "center",
    "col",
    "colgroup",
    "dd",
    "details",
    "dir",
    "div",
    "dl",
    "dt",
    "embed",
    "fieldset",
    "figcaption",
    "figure",
    "footer",
    "form",
    "frame",
    "frameset",
    "h1",
    "h2",
    "h3",
    "h4",
    "h5",
    "h6",
    "head",
    "header",
    "hgroup",
    "hr",
    "html",
    "iframe",
    "img",
    "input",
    "keygen",
    "li",
    "link",
    "listing",
    "main",
    "marquee",
    "menu",
    "meta",
    "nav",
    "noembed",
    "noframes",
    "noscript",
    "object",
    "ol",
    "p",
    "param",
    "plaintext",
    "pre",
    "script",
    "search",
    "section",
    "select",
    "source",
    "style",
    "summary",
    "table",
    "tbody",
    "td",
    "template",
    "textarea",
    "tfoot",
    "th",
    "thead",
    "title",
    "tr",
    "track",
    "ul",
    "wbr",
    "xmp",
    // FIXME: Implement MathML mi
    // FIXME: Implement MathML mo
    // FIXME: Implement MathML mn
    // FIXME: Implement MathML ms
    // FIXME: Implement MathML mtext
    // FIXME: Implement MathML annotation-xml
    // FIXME: Implement SVG foreignObject
    // FIXME: Implement SVG desc
    // FIXME: Implement SVG title
];

pub static BASE_SCOPE_TAGS: &[&str] = &[
    "applet",
    "caption",
    "html",
    "table",
    "td",
    "th",
    "marquee",
    "object",
    "template",
    "mi",
    "mo",
    "mn",
    "ms",
    "mtext",
    "annotation-xml",
    "foreignObject",
    "desc",
    "title",
];

/// https://html.spec.whatwg.org/multipage/parsing.html#the-stack-of-open-elements
#[derive(Debug, Clone, PartialEq)]
struct StackOfOpenElements {
    elements: Vec<NodeId>,
}

impl StackOfOpenElements {
    pub fn new() -> Self {
        Self { elements: vec![] }
    }

    pub fn is_empty(&self) -> bool {
        self.elements.is_empty()
    }

    pub fn contains(&self, node: NodeId) -> bool {
        self.elements.contains(&node)
    }

    /// https://html.spec.whatwg.org/multipage/parsing.html#current-node
    pub fn current_node(&self) -> NodeId {
        *self
            .elements
            .last()
            .expect("Should always have a value. If not the parser should have finished.")
    }

    /// https://html.spec.whatwg.org/multipage/parsing.html#adjusted-current-node
    fn adjusted_current_node(&self) -> NodeId {
        // TODO: The adjusted current node is the context element
        // if the parser was created as part of the
        // HTML fragment parsing algorithm and the stack of open elements
        // has only one element in it (fragment case);

        // otherwise, the adjusted current node is the current node.
        self.current_node()
    }

    pub fn push(&mut self, element: NodeId) {
        self.elements.push(element);
    }

    pub fn pop(&mut self) -> Option<NodeId> {
        self.elements.pop()
    }

    pub fn pop_until_element_with_tag_name(&mut self, arena: &NodeArena, tag_name: &str) {
        while let Some(node) = self.elements.pop() {
            if arena.get_node(node).is_element_with_tag_name(tag_name) {
                break;
            }
        }
    }

    pub fn has_element_in_specific_scope(
        &self,
        arena: &NodeArena,
        target_node: &str,
        tag_names: &[&str],
    ) -> bool {
        // 1. Initialize node to be the current node (the bottommost node of the stack).
        for node in self.elements.iter().rev() {
            let node = arena.get_node(*node);

            // 2. If node is the target node, terminate in a match state.
            if node.is_element_with_tag_name(target_node) {
                return true;
            }

            // 3. Otherwise, if node is one of the element types in list, terminate in
            // a failure state.
            for tag_name in tag_names.iter() {
                if node.is_element_with_tag_name(tag_name) {
                    return false;
                }
            }

            // 4. Otherwise, set node to the previous entry in the stack of open
            // elements and return to step 2. (This will never fail, since the
            // loop will always terminate in the previous step if
            // the top of the stack — an html element — is reached.)
        }

        unreachable!()
    }

    pub fn has_element_in_scope(&self, arena: &NodeArena, element: &str) -> bool {
        self.has_element_in_specific_scope(arena, element, &BASE_SCOPE_TAGS)
    }

    /// https://html.spec.whatwg.org/multipage/parsing.html#has-an-element-in-list-scope
    pub fn has_element_in_list_scope(&self, arena: &NodeArena, element: &str) -> bool {
        self.has_element_in_specific_scope(
            arena,
            element,
            &[BASE_SCOPE_TAGS, &["ol", "ul"]].concat(),
        )
    }

    /// https://html.spec.whatwg.org/multipage/parsing.html#has-an-element-in-button-scope
    pub fn has_element_in_button_scope(&self, arena: &NodeArena, element: &str) -> bool {
        self.has_element_in_specific_scope(arena, element, &[BASE_SCOPE_TAGS, &["button"]].concat())
    }

    /// https://html.spec.whatwg.org/multipage/parsing.html#has-an-element-in-table-scope
    pub fn has_element_in_table_scope(&self, arena: &NodeArena, element: &str) -> bool {
        self.has_element_in_specific_scope(
            arena,
            element,
            &[BASE_SCOPE_TAGS, &["html", "table", "template"]].concat(),
        )
    }

    pub fn insert_immediately_below(&mut self, element: NodeId, target: NodeId) {
        if let Some(index) = self.elements.iter().position(|e| e == &target) {
            self.elements.insert(index + 1, element);
        }
    }

    pub fn replace(&mut self, target: NodeId, replacement: NodeId) {
        if let Some(index) = self.elements.iter().position(|e| e == &target) {
            self.elements[index] = replacement;
        }
    }

    pub fn remove_element(&mut self, element: NodeId) {
        if let Some(index) = self.elements.iter().position(|e| e == &element) {
            self.elements.remove(index);
        }
    }

    pub fn element_immediately_above(&self, target: NodeId) -> Option<NodeId> {
        let mut found = false;
        for element in self.elements.iter().rev() {
            if *element == target {
                found = true;
            } else if found {
                return Some(*element);
            }
        }
        None
    }

    pub fn topmost_special_node_below(&self, arena: &NodeArena, target: NodeId) -> Option<NodeId> {
        let mut best = None;
        for element in self.elements.iter().rev() {
            if *element == target {
                break;
            }
            if arena
                .get_node(*element)
                .is_element_with_one_of_tag_names(&SPECIAL_TAGS)
            {
                best = Some(*element);
            }
        }
        best
    }
}

#[derive(Debug, Clone, PartialEq)]
enum ActiveFormattingElement {
    Marker,
    Element(NodeId),
}

#[derive(PartialEq, Eq, Debug, Clone, Copy)]
pub enum FormattingElementPosition {
    End,
    LastMarkerOrElseStart,
}

/// https://html.spec.whatwg.org/multipage/parsing.html#list-of-active-formatting-elements
#[derive(Debug, Clone, PartialEq)]
struct ActiveFormattingElements {
    elements: Vec<ActiveFormattingElement>,
}

impl ActiveFormattingElements {
    pub fn new() -> Self {
        Self { elements: vec![] }
    }

    /// https://html.spec.whatwg.org/multipage/parsing.html#reconstruct-the-active-formatting-elements
    pub fn reconstruct(&mut self, open_elements: &StackOfOpenElements) {
        // If there are no entries in the list of active formatting elements,
        // then there is nothing to reconstruct; stop this algorithm.
        if self.elements.is_empty() {
            return;
        }

        // If the last (most recently added) entry in the list of active
        // formatting elements is a marker, or if it is an element that is in
        // the stack of open elements, then there is nothing to reconstruct;
        // stop this algorithm.
        match self.elements.last().unwrap() {
            ActiveFormattingElement::Marker => return,
            ActiveFormattingElement::Element(element) if open_elements.contains(*element) => {
                return;
            }
            _ => {}
        }

        todo!("Fully implement reconstructing active formatting elements");

        // TODO: Let entry be the last (most recently added) element in the list
        // of active formatting elements.

        // TODO: Rewind: If there are no entries before entry in the list of
        // active formatting elements, then jump to the step labeled
        // create.

        // TODO: Let entry be the entry one earlier than entry in the list of
        // active formatting elements.

        // TODO: If entry is neither a marker nor an element that is also in the
        // stack of open elements, go to the step labeled rewind.

        // TODO: Advance: Let entry be the element one later than entry in the
        // list of active formatting elements.

        // TODO: Create: Insert an HTML element for the token for which the
        // element entry was created, to obtain new element.

        // TODO: Replace the entry for entry in the list with an entry for new
        // element.

        // TODO: If the entry for new element in the list of active formatting
        // elements is not the last entry in the list, return to the step
        // labeled advance.
    }

    pub fn push(&mut self, element: ActiveFormattingElement) {
        self.elements.push(element);
    }

    pub fn first_index_of(&self, target: NodeId) -> Option<usize> {
        self.elements
            .iter()
            .position(|e| *e == ActiveFormattingElement::Element(target))
    }

    pub fn replace(&mut self, target: NodeId, replacement: NodeId) {
        if let Some(index) = self
            .elements
            .iter()
            .position(|e| *e == ActiveFormattingElement::Element(target))
        {
            self.elements[index] = ActiveFormattingElement::Element(replacement);
        }
    }

    pub fn insert(&mut self, index: usize, element: NodeId) {
        self.elements
            .insert(index, ActiveFormattingElement::Element(element));
    }

    pub fn remove(&mut self, element: NodeId) {
        if let Some(index) = self
            .elements
            .iter()
            .position(|e| *e == ActiveFormattingElement::Element(element))
        {
            self.elements.remove(index);
        }
    }

    pub fn len(&self) -> usize {
        self.elements.len()
    }

    pub fn last_element_with_tag_name_before_marker(
        &self,
        arena: &NodeArena,
        tag_name: &str,
    ) -> Option<NodeId> {
        for element in self.elements.iter().rev() {
            if matches!(element, ActiveFormattingElement::Marker) {
                break;
            }
            if let ActiveFormattingElement::Element(element) = element {
                if arena.get_node(*element).is_element_with_tag_name(tag_name) {
                    return Some(*element);
                }
            }
        }
        None
    }

    pub fn contains(&self, target: NodeId) -> bool {
        self.elements.iter().any(|element| {
            if let ActiveFormattingElement::Element(element) = element {
                return *element == target;
            }
            false
        })
    }

    pub fn contains_element_between(
        &self,
        arena: &NodeArena,
        start: FormattingElementPosition,
        end: FormattingElementPosition,
        tag_name: &str,
    ) -> bool {
        if let Some(start) = self.index_from_position(start) {
            if let Some(end) = self.index_from_position(end) {
                for i in start..end {
                    if let Some(ActiveFormattingElement::Element(element)) = self.elements.get(i) {
                        if arena
                            .get_node(*element)
                            .is_element_with_one_of_tag_names(&[tag_name])
                        {
                            return true;
                        }
                    }
                }
            }
        }
        false
    }

    fn index_from_position(&self, position: FormattingElementPosition) -> Option<usize> {
        match position {
            FormattingElementPosition::End => Some(self.len().saturating_sub(1)),
            FormattingElementPosition::LastMarkerOrElseStart => self
                .elements
                .iter()
                .rev()
                .enumerate()
                .find(|(_, element)| matches!(element, ActiveFormattingElement::Marker))
                .map(|element| element.0),
        }
    }
}
