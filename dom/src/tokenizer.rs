#![allow(dead_code)]

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum State {
    Data,
    RcData,
    RawText,
    ScriptData,
    PlainText,
    TagOpen,
    EndTagOpen,
    TagName,
    RcDataLessThanSign,
    RcDataEndTagOpen,
    RcDataEndTagName,
    RawTextLessThanSign,
    RawTextEndTagOpen,
    RawTextEndTagName,
    ScriptDataLessThanSign,
    ScriptDataEndTagOpen,
    ScriptDataEndTagName,
    ScriptDataEscapeStart,
    ScriptDataEscapeStartDash,
    ScriptDataEscaped,
    ScriptDataEscapedDash,
    ScriptDataEscapedDashDash,
    ScriptDataEscapedLessThanSign,
    ScriptDataEscapedEndTagOpen,
    ScriptDataEscapedEndTagName,
    ScriptDataDoubleEscapeStart,
    ScriptDataDoubleEscaped,
    ScriptDataDoubleEscapedDash,
    ScriptDataDoubleEscapedDashDash,
    ScriptDataDoubleEscapedLessThanSign,
    ScriptDataDoubleEscapeEnd,
    BeforeAttributeName,
    AttributeName,
    AfterAttributeName,
    BeforeAttributeValue,
    AttributeValueDoubleQuoted,
    AttributeValueSingleQuoted,
    AttributeValueUnquoted,
    AfterAttributeValueQuoted,
    SelfClosingStartTag,
    BogusComment,
    MarkupDeclarationOpen,
    CommentStart,
    CommentStartDash,
    Comment,
    CommentLessThanSign,
    CommentLessThanSignBang,
    CommentLessThanSignBangDash,
    CommentLessThanSignBangDashDash,
    CommentEndDash,
    CommentEnd,
    CommentEndBang,
    Doctype,
    BeforeDoctypeName,
    DoctypeName,
    AfterDoctypeName,
    AfterDoctypePublicKeyword,
    BeforeDoctypePublicIdentifier,
    DoctypePublicIdentifierDoubleQuoted,
    DoctypePublicIdentifierSingleQuoted,
    AfterDoctypePublicIdentifier,
    BetweenDoctypePublicAndSystemIdentifiers,
    AfterDoctypeSystemKeyword,
    BeforeDoctypeSystemIdentifier,
    DoctypeSystemIdentifierDoubleQuoted,
    DoctypeSystemIdentifierSingleQuoted,
    AfterDoctypeSystemIdentifier,
    BogusDoctype,
    CDataSection,
    CDataSectionBracket,
    CDataSectionEnd,
    CharacterReference,
    NamedCharacterReference,
    AmbiguousAmpersand,
    NumericCharacterReference,
    HexadecimalCharacterReferenceStart,
    DecimalCharacterReferenceStart,
    HexadecimalCharacterReference,
    DecimalCharacterReference,
    NumericCharacterReferenceEnd,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Attribute {
    pub name: String,
    pub value: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Token {
    EndOfFile,
    Character(char),
    Tag {
        start: bool,
        tag_name: String,
        attributes: Vec<Attribute>,
    },
    Comment {
        data: String,
    },
    Doctype {
        name: String,
        public_identifier: Option<String>,
        system_identifier: Option<String>,
    },
}

impl Token {
    pub fn is_start_tag_with_name(&self, names: &[&str]) -> bool {
        self.is_start_tag() && self.is_tag_with_name(names)
    }

    pub fn is_end_tag_with_name(&self, names: &[&str]) -> bool {
        self.is_end_tag() && self.is_tag_with_name(names)
    }

    pub fn is_tag_with_name(&self, names: &[&str]) -> bool {
        if let Token::Tag { tag_name, .. } = self {
            return names.contains(&tag_name.as_str());
        }
        false
    }

    pub fn is_start_tag(&self) -> bool {
        if let Token::Tag { start, .. } = self {
            return *start;
        }
        false
    }

    pub fn is_end_tag(&self) -> bool {
        !self.is_start_tag()
    }
}

macro_rules! null {
    () => {
        Some('\0')
    };
}

macro_rules! eof {
    () => {
        None
    };
}

macro_rules! ascii_upper_alpha {
    () => {
        Some('A'..='Z')
    };
}

macro_rules! ascii_alpha {
    () => {
        Some('a'..='z') | ascii_upper_alpha!()
    };
}

macro_rules! whitespace {
    () => {
        Some('\u{0009}') | Some('\u{000A}') | Some('\u{000C}') | Some('\u{0020}')
    };
}

#[derive(Debug, Clone)]
pub struct Tokenizer<'input> {
    html: &'input str,
    state: State,
    return_state: State,
    tokens: Vec<Token>,
    current_token: Option<Token>,
    insertion_point: usize,
}

impl<'input> Tokenizer<'input> {
    pub fn new(html: &'input str) -> Self {
        Self {
            html,
            state: State::Data,
            return_state: State::Data,
            tokens: vec![],
            current_token: None,
            insertion_point: 0,
        }
    }

    pub fn peek(&mut self) -> Option<&Token> {
        self.tokens.last()
    }

    pub fn next(&mut self) -> Option<Token> {
        let mut emitted_token: Option<Token> = None;

        macro_rules! emit_token {
            ($token:expr) => {
                emitted_token = Some($token)
            };
        }

        macro_rules! emit_current_token {
            () => {
                if let Some(token) = self.current_token.take() {
                    emit_token!(token);
                    self.current_token = None;
                }
            };
        }

        while emitted_token.is_none() {
            match self.state {
                State::Data => match self.consume_next_input_character() {
                    Some('&') => {
                        self.set_return_state(State::Data);
                        self.switch_to(State::CharacterReference);
                    }
                    Some('<') => {
                        self.switch_to(State::TagOpen);
                    }
                    null!() => {
                        todo!("This is an unexpected-null-character parse error. Emit the current input character as a character token.");
                    }
                    eof!() => {
                        emit_token!(Token::EndOfFile);
                    }
                    Some(anything_else) => {
                        emit_token!(Token::Character(anything_else));
                    }
                },
                State::RcData => todo!("RcData"),
                State::RawText => todo!("RawText"),
                State::ScriptData => todo!("ScriptData"),
                State::PlainText => todo!("PlainText"),
                State::TagOpen => match self.consume_next_input_character() {
                    Some('!') => {
                        self.switch_to(State::MarkupDeclarationOpen);
                    }
                    Some('/') => {
                        self.switch_to(State::EndTagOpen);
                    }
                    ascii_alpha!() => {
                        self.set_current_token(Token::Tag {
                            start: true,
                            tag_name: "".to_string(),
                            attributes: vec![],
                        });
                        self.reconsume_in_state(State::TagName);
                    }
                    Some('?') => {
                        todo!("This is an unexpected-question-mark-instead-of-tag-name parse error. Create a comment token whose data is the empty string. Reconsume in the bogus comment state.");
                    }
                    eof!() => {
                        todo!("This is an eof-before-tag-name parse error. Emit a U+003C LESS-THAN SIGN character token and an end-of-file token.");
                    }
                    Some(_) => {
                        todo!("This is an invalid-first-character-of-tag-name parse error. Emit a U+003C LESS-THAN SIGN character token. Reconsume in the data state.");
                    }
                },
                State::EndTagOpen => {
                    match self.consume_next_input_character() {
                        ascii_alpha!() => {
                            self.set_current_token(Token::Tag {
                                start: false,
                                tag_name: "".to_string(),
                                attributes: vec![],
                            });
                            self.reconsume_in_state(State::TagName);
                        }
                        Some('>') => {
                            todo!("This is a missing-end-tag-name parse error. Switch to the data state.");
                        }
                        eof!() => {
                            todo!("This is an eof-before-tag-name parse error. Emit a U+003C LESS-THAN SIGN character token, a U+002F SOLIDUS character token and an end-of-file token.");
                        }
                        Some(_) => {
                            todo!("This is an invalid-first-character-of-tag-name parse error. Create a comment token whose data is the empty string. Reconsume in the bogus comment state.");
                        }
                    }
                }
                State::TagName => match self.consume_next_input_character() {
                    whitespace!() => {
                        self.switch_to(State::BeforeAttributeName);
                    }
                    Some('/') => {
                        self.switch_to(State::SelfClosingStartTag);
                    }
                    Some('>') => {
                        self.switch_to(State::Data);
                        emit_current_token!();
                    }
                    null!() => {
                        todo!("This is an unexpected-null-character parse error. Append a U+FFFD REPLACEMENT CHARACTER character to the current tag token's tag name.");
                    }
                    eof!() => {
                        todo!("This is an eof-in-tag parse error. Emit an end-of-file token.");
                    }
                    Some(anything_else) => {
                        // ASCII upper alpha:
                        // Append the lowercase version of the current input character
                        // (add 0x0020 to the character's code point)
                        // to the current tag token's tag name.
                        let character = anything_else.to_ascii_lowercase();

                        if let Some(Token::Tag { tag_name, .. }) = self.current_token.as_mut() {
                            tag_name.push(character);
                        }
                    }
                },
                State::RcDataLessThanSign => todo!("RcDataLessThanSign"),
                State::RcDataEndTagOpen => todo!("RcDataEndTagOpen"),
                State::RcDataEndTagName => todo!("RcDataEndTagName"),
                State::RawTextLessThanSign => todo!("RawTextLessThanSign"),
                State::RawTextEndTagOpen => todo!("RawTextEndTagOpen"),
                State::RawTextEndTagName => todo!("RawTextEndTagName"),
                State::ScriptDataLessThanSign => todo!("ScriptDataLessThanSign"),
                State::ScriptDataEndTagOpen => todo!("ScriptDataEndTagOpen"),
                State::ScriptDataEndTagName => todo!("ScriptDataEndTagName"),
                State::ScriptDataEscapeStart => todo!("ScriptDataEscapeStart"),
                State::ScriptDataEscapeStartDash => todo!("ScriptDataEscapeStartDash"),
                State::ScriptDataEscaped => todo!("ScriptDataEscaped"),
                State::ScriptDataEscapedDash => todo!("ScriptDataEscapedDash"),
                State::ScriptDataEscapedDashDash => todo!("ScriptDataEscapedDashDash"),
                State::ScriptDataEscapedLessThanSign => todo!("ScriptDataEscapedLessThanSign"),
                State::ScriptDataEscapedEndTagOpen => todo!("ScriptDataEscapedEndTagOpen"),
                State::ScriptDataEscapedEndTagName => todo!("ScriptDataEscapedEndTagName"),
                State::ScriptDataDoubleEscapeStart => todo!("ScriptDataDoubleEscapeStart"),
                State::ScriptDataDoubleEscaped => todo!("ScriptDataDoubleEscaped"),
                State::ScriptDataDoubleEscapedDash => todo!("ScriptDataDoubleEscapedDash"),
                State::ScriptDataDoubleEscapedDashDash => todo!("ScriptDataDoubleEscapedDashDash"),
                State::ScriptDataDoubleEscapedLessThanSign => {
                    todo!("ScriptDataDoubleEscapedLessThanSign")
                }
                State::ScriptDataDoubleEscapeEnd => todo!("ScriptDataDoubleEscapeEnd"),
                State::BeforeAttributeName => match self.consume_next_input_character() {
                    whitespace!() => {}
                    Some('/') | Some('<') | eof!() => {
                        self.reconsume_in_state(State::AfterAttributeName);
                    }
                    Some('=') => {
                        todo!("This is an unexpected-equals-sign-before-attribute-name parse error. Start a new attribute in the current tag token. Set that attribute's name to the current input character, and its value to the empty string. Switch to the attribute name state.");
                    }
                    Some(_) => {
                        if let Some(Token::Tag { attributes, .. }) = &mut self.current_token {
                            attributes.push(Attribute {
                                name: "".to_string(),
                                value: "".to_string(),
                            })
                        }
                        self.reconsume_in_state(State::AttributeName);
                    }
                },
                State::AttributeName => match self.consume_next_input_character() {
                    whitespace!() | Some('/') | Some('>') | eof!() => {
                        self.reconsume_in_state(State::AfterAttributeName);
                    }
                    Some('=') => {
                        self.switch_to(State::BeforeAttributeValue);
                    }
                    null!() => {
                        todo!("This is an unexpected-null-character parse error. Append a U+FFFD REPLACEMENT CHARACTER character to the current attribute's name.");
                    }
                    Some('"') | Some('\'') | Some('<') => {
                        todo!("This is an unexpected-character-in-attribute-name parse error. Treat it as per the 'anything else' entry below.");
                    }
                    Some(anything_else) => {
                        if let Some(Token::Tag { attributes, .. }) = &mut self.current_token {
                            if let Some(attribute) = attributes.last_mut() {
                                attribute.name.push(anything_else);
                            }
                        }
                    }
                },
                State::AfterAttributeName => todo!("AfterAttributeName"),
                State::BeforeAttributeValue => match self.consume_next_input_character() {
                    whitespace!() => {}
                    Some('"') => {
                        self.switch_to(State::AttributeValueDoubleQuoted);
                    }
                    Some('\'') => {
                        self.switch_to(State::AttributeValueSingleQuoted);
                    }
                    Some('>') => {
                        todo!("This is a missing-attribute-value parse error. Switch to the data state. Emit the current tag token.");
                    }
                    Some(_) | eof!() => {
                        self.reconsume_in_state(State::AttributeValueUnquoted);
                    }
                },
                State::AttributeValueDoubleQuoted => match self.consume_next_input_character() {
                    Some('"') => {
                        self.switch_to(State::AfterAttributeValueQuoted);
                    }
                    Some('&') => {
                        self.set_return_state(State::AttributeValueDoubleQuoted);
                        self.switch_to(State::CharacterReference);
                    }
                    null!() => {
                        todo!("This is an unexpected-null-character parse error. Append a U+FFFD REPLACEMENT CHARACTER character to the current attribute's value.");
                    }
                    eof!() => {
                        todo!("This is an eof-in-tag parse error. Emit an end-of-file token.");
                    }
                    Some(anything_else) => {
                        if let Some(Token::Tag { attributes, .. }) = &mut self.current_token {
                            if let Some(attribute) = attributes.last_mut() {
                                attribute.value.push(anything_else);
                            }
                        }
                    }
                },
                State::AttributeValueSingleQuoted => todo!("AttributeValueSingleQuoted"),
                State::AttributeValueUnquoted => todo!("AttributeValueUnquoted"),
                State::AfterAttributeValueQuoted => match self.consume_next_input_character() {
                    whitespace!() => {
                        self.switch_to(State::BeforeAttributeName);
                    }
                    Some('/') => {
                        self.switch_to(State::SelfClosingStartTag);
                    }
                    Some('>') => {
                        self.switch_to(State::Data);
                        emit_current_token!();
                    }
                    eof!() => {
                        todo!("This is an eof-in-tag parse error. Emit an end-of-file token.");
                    }
                    Some(_) => {
                        todo!("This is a missing-whitespace-between-attributes parse error. Reconsume in the before attribute name state.");
                    }
                },
                State::SelfClosingStartTag => todo!("SelfClosingStartTag"),
                State::BogusComment => todo!("BogusComment"),
                State::MarkupDeclarationOpen => {
                    if self.next_few_input_characters_are("--", false) {
                        self.consume_word("--");
                        self.set_current_token(Token::Comment {
                            data: "".to_string(),
                        });
                        self.switch_to(State::CommentStart);
                    } else if self.next_few_input_characters_are("DOCTYPE", true) {
                        self.consume_word("DOCTYPE");
                        self.switch_to(State::Doctype);
                    }
                }
                State::CommentStart => todo!("CommentStart"),
                State::CommentStartDash => todo!("CommentStartDash"),
                State::Comment => todo!("Comment"),
                State::CommentLessThanSign => todo!("CommentLessThanSign"),
                State::CommentLessThanSignBang => todo!("CommentLessThanSignBang"),
                State::CommentLessThanSignBangDash => todo!("CommentLessThanSignBangDash"),
                State::CommentLessThanSignBangDashDash => todo!("CommentLessThanSignBangDashDash"),
                State::CommentEndDash => todo!("CommentEndDash"),
                State::CommentEnd => todo!("CommentEnd"),
                State::CommentEndBang => todo!("CommentEndBang"),
                State::Doctype => match self.consume_next_input_character() {
                    whitespace!() => {
                        self.switch_to(State::BeforeDoctypeName);
                    }
                    Some('>') => {
                        self.reconsume_in_state(State::BeforeDoctypeName);
                    }
                    eof!() => {
                        todo!("This is an eof-in-doctype parse error. Create a new DOCTYPE token. Set its force-quirks flag to on. Emit the current token. Emit an end-of-file token.");
                    }
                    _ => {
                        todo!("This is a missing-whitespace-before-doctype-name parse error. Reconsume in the before DOCTYPE name state.");
                    }
                },
                State::BeforeDoctypeName => match self.consume_next_input_character() {
                    whitespace!() => {}
                    ascii_upper_alpha!() => {
                        self.set_current_token(Token::Doctype {
                            name: self
                                .current_input_character()
                                .unwrap()
                                .to_ascii_lowercase()
                                .to_string(),
                            public_identifier: None,
                            system_identifier: None,
                        });
                        self.switch_to(State::DoctypeName);
                    }
                    null!() => {
                        todo!("This is an unexpected-null-character parse error. Create a new DOCTYPE token. Set the token's name to a U+FFFD REPLACEMENT CHARACTER character. Switch to the DOCTYPE name state.");
                    }
                    Some('>') => {
                        todo!("This is a missing-doctype-name parse error. Create a new DOCTYPE token. Set its force-quirks flag to on. Switch to the data state. Emit the current token.")
                    }
                    eof!() => {
                        todo!("This is an eof-in-doctype parse error. Create a new DOCTYPE token. Set its force-quirks flag to on. Emit the current token. Emit an end-of-file token.");
                    }
                    Some(char) => {
                        self.set_current_token(Token::Doctype {
                            name: char.to_string(),
                            public_identifier: None,
                            system_identifier: None,
                        });
                        self.switch_to(State::DoctypeName);
                    }
                },
                State::DoctypeName => match self.consume_next_input_character() {
                    whitespace!() => {
                        self.switch_to(State::AfterDoctypeName);
                    }
                    Some('>') => {
                        self.switch_to(State::Data);
                        emit_current_token!();
                    }
                    ascii_upper_alpha!() => {
                        let char = self.current_input_character().unwrap();
                        if let Some(Token::Doctype { name, .. }) = &mut self.current_token {
                            name.push(char.to_ascii_lowercase());
                        }
                    }
                    null!() => {
                        todo!("This is an unexpected-null-character parse error. Append a U+FFFD REPLACEMENT CHARACTER character to the current DOCTYPE token's name.");
                    }
                    eof!() => {
                        todo!("This is an eof-in-doctype parse error. Set the current DOCTYPE token's force-quirks flag to on. Emit the current DOCTYPE token. Emit an end-of-file token.");
                    }
                    Some(char) => {
                        if let Some(Token::Doctype { name, .. }) = &mut self.current_token {
                            name.push(char);
                        }
                    }
                },
                State::AfterDoctypeName => match self.consume_next_input_character() {
                    whitespace!() => {}
                    Some('>') => {
                        self.switch_to(State::Data);
                        emit_current_token!();
                    }
                    eof!() => {
                        todo!("This is an eof-in-doctype parse error. Set the current DOCTYPE token's force-quirks flag to on. Emit the current DOCTYPE token. Emit an end-of-file token.");
                    }
                    _ => {
                        todo!();
                    }
                },
                State::AfterDoctypePublicKeyword => todo!("AfterDoctypePublicKeyword"),
                State::BeforeDoctypePublicIdentifier => todo!("BeforeDoctypePublicIdentifier"),
                State::DoctypePublicIdentifierDoubleQuoted => {
                    todo!("DoctypePublicIdentifierDoubleQuoted")
                }
                State::DoctypePublicIdentifierSingleQuoted => {
                    todo!("DoctypePublicIdentifierSingleQuoted")
                }
                State::AfterDoctypePublicIdentifier => todo!("AfterDoctypePublicIdentifier"),
                State::BetweenDoctypePublicAndSystemIdentifiers => {
                    todo!("BetweenDoctypePublicAndSystemIdentifiers")
                }
                State::AfterDoctypeSystemKeyword => todo!("AfterDoctypeSystemKeyword"),
                State::BeforeDoctypeSystemIdentifier => todo!("BeforeDoctypeSystemIdentifier"),
                State::DoctypeSystemIdentifierDoubleQuoted => {
                    todo!("DoctypeSystemIdentifierDoubleQuoted")
                }
                State::DoctypeSystemIdentifierSingleQuoted => {
                    todo!("DoctypeSystemIdentifierSingleQuoted")
                }
                State::AfterDoctypeSystemIdentifier => todo!("AfterDoctypeSystemIdentifier"),
                State::BogusDoctype => todo!("BogusDoctype"),
                State::CDataSection => todo!("CDataSection"),
                State::CDataSectionBracket => todo!("CDataSectionBracket"),
                State::CDataSectionEnd => todo!("CDataSectionEnd"),
                State::CharacterReference => todo!("CharacterReference"),
                State::NamedCharacterReference => todo!("NamedCharacterReference"),
                State::AmbiguousAmpersand => todo!("AmbiguousAmpersand"),
                State::NumericCharacterReference => todo!("NumericCharacterReference"),
                State::HexadecimalCharacterReferenceStart => {
                    todo!("HexadecimalCharacterReferenceStart")
                }
                State::DecimalCharacterReferenceStart => todo!("DecimalCharacterReferenceStart"),
                State::HexadecimalCharacterReference => todo!("HexadecimalCharacterReference"),
                State::DecimalCharacterReference => todo!("DecimalCharacterReference"),
                State::NumericCharacterReferenceEnd => todo!("NumericCharacterReferenceEnd"),
            }
        }

        if let Some(emitted_token) = emitted_token {
            self.tokens.push(emitted_token);
        }

        self.peek().cloned()
    }

    fn current_input_character(&self) -> Option<char> {
        self.html.chars().nth(self.insertion_point)
    }

    fn next_input_character(&mut self) -> Option<char> {
        self.html.chars().nth(self.insertion_point + 1)
    }

    fn next_few_input_characters_are(&self, word: &str, case_sensitive: bool) -> bool {
        self.html[self.insertion_point..]
            .chars()
            .zip(word.chars())
            .all(|(a, b)| {
                if case_sensitive {
                    a == b
                } else {
                    a.eq_ignore_ascii_case(&b)
                }
            })
    }

    fn switch_to(&mut self, state: State) {
        self.state = state;
    }

    fn set_return_state(&mut self, state: State) {
        self.return_state = state;
    }

    fn reconsume_in_state(&mut self, state: State) {
        self.insertion_point -= 1;
        self.switch_to(state);
    }

    fn set_current_token(&mut self, token: Token) {
        self.current_token = Some(token);
    }

    fn consume_next_input_character(&mut self) -> Option<char> {
        let char = self.current_input_character();
        self.insertion_point += 1;
        char
    }

    fn consume_word(&mut self, word: &str) {
        self.insertion_point += word.len();
    }
}
