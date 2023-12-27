use crate::arena::{NodeArena, NodeId};
use crate::node::Node;
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
    after: Option<NodeId>,
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
        arena.insert(element, self.parent, self.after)

        // TODO: If the parser was not created as part of the HTML fragment
        // parsing algorithm, then pop the element queue from element's
        // relevant agent's custom element reactions stack, and invoke
        // custom element reactions in that queue.
    }
}

#[derive(Debug)]
pub struct Parser<'input, 'arena> {
    arena: &'arena mut NodeArena,
    tokenizer: tokenizer::Tokenizer<'input>,
    insertion_mode: InsertionMode,
    should_reprocess_token: bool,
    document: NodeId,
    open_elements: Vec<NodeId>,
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
            should_reprocess_token: false,
            document: arena.create_node(Node::create_document()),
            open_elements: vec![],
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

                    self.switch_insertion_mode_and_reprocess_token(InsertionMode::BeforeHtml);
                }
            },
            InsertionMode::BeforeHtml => {
                match token {
                    Token::Doctype => {
                        todo!("Parse error. Ignore the token.");
                    }
                    Token::Comment => {
                        todo!("Insert a comment as the last child of the Document object.");
                    }
                    whitespace!() => {}
                    Token::Tag { .. } if token.is_start_tag_with_name(&["html"]) => {
                        let html_element =
                            self.create_element_for_token(token, Namespace::Html, self.document);
                        self.arena.append(html_element, self.document);
                        self.open_elements.push(html_element);
                        self.switch_insertion_mode(InsertionMode::BeforeHead);
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
                }
            }
            InsertionMode::BeforeHead => match token {
                whitespace!() => {}
                Token::Comment => {
                    todo!("Insert a comment.");
                }
                Token::Doctype => {
                    todo!("Parse error. Ignore the token.");
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
                    todo!("Parse error. Ignore the token.")
                }
                _ => {
                    self.insert_html_element(&Token::Tag {
                        start: true,
                        tag_name: "body".to_string(),
                        attributes: vec![],
                    });
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
        self.open_elements.push(element);

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
            None => self.current_node(),
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
                after: None,
            }
        };

        // TODO: If the adjusted insertion location is inside a template
        // element, let it instead be inside the template element's template
        // contents, after its last child (if any).

        // Return the adjusted insertion location.
        adjusted_insertion_location
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
        // if the parser was created as part of the
        // HTML fragment parsing algorithm and the stack of open elements
        // has only one element in it (fragment case);

        // otherwise, the adjusted current node is the current node.
        self.current_node()
    }

    fn is_in_foreign_content(&self, token: &Token) -> bool {
        // If the stack of open elements is empty
        if self.stack_of_open_elements_is_empty() {
            return false;
        }

        let acn = self.arena.get_node(self.adjusted_current_node());

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
}
