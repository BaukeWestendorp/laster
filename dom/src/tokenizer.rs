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
pub enum Token {
    EndOfFile,
    Character(char),
    Tag {
        is_start_tag: bool,
        tag_name: String,
    },
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

macro_rules! ascii_alpha {
    () => {
        Some('a'..='z') | Some('A'..='Z')
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

    pub fn tokenize(&mut self) -> Vec<Token> {
        while self.tokens.last() != Some(&Token::EndOfFile) {
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
                        todo!("This is an unexpected-null-character parse error. Emit the current input character as a character token.")
                    }
                    eof!() => {
                        self.emit_token(Token::EndOfFile);
                    }
                    Some(anything_else) => {
                        self.tokens.push(Token::Character(anything_else));
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
                            is_start_tag: true,
                            tag_name: "".to_string(),
                        });
                        self.reconsume_in_state(State::TagName);
                    }
                    Some('?') => {
                        todo!("This is an unexpected-question-mark-instead-of-tag-name parse error. Create a comment token whose data is the empty string. Reconsume in the bogus comment state.")
                    }
                    eof!() => {
                        todo!("This is an eof-before-tag-name parse error. Emit a U+003C LESS-THAN SIGN character token and an end-of-file token.")
                    }
                    Some(_) => {
                        todo!("This is an invalid-first-character-of-tag-name parse error. Emit a U+003C LESS-THAN SIGN character token. Reconsume in the data state.")
                    }
                },
                State::EndTagOpen => {
                    match self.consume_next_input_character() {
                        ascii_alpha!() => {
                            self.set_current_token(Token::Tag {
                                is_start_tag: false,
                                tag_name: "".to_string(),
                            });
                            self.reconsume_in_state(State::TagName);
                        }
                        Some('>') => {
                            todo!("This is a missing-end-tag-name parse error. Switch to the data state.")
                        }
                        eof!() => {
                            todo!("This is an eof-before-tag-name parse error. Emit a U+003C LESS-THAN SIGN character token, a U+002F SOLIDUS character token and an end-of-file token.")
                        }
                        Some(_) => {
                            todo!("This is an invalid-first-character-of-tag-name parse error. Create a comment token whose data is the empty string. Reconsume in the bogus comment state.")
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
                        self.emit_current_token();
                    }
                    null!() => {
                        todo!("This is an unexpected-null-character parse error. Append a U+FFFD REPLACEMENT CHARACTER character to the current tag token's tag name.")
                    }
                    eof!() => {
                        todo!("This is an eof-in-tag parse error. Emit an end-of-file token.")
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
                State::BeforeAttributeName => todo!("BeforeAttributeName"),
                State::AttributeName => todo!("AttributeName"),
                State::AfterAttributeName => todo!("AfterAttributeName"),
                State::BeforeAttributeValue => todo!("BeforeAttributeValue"),
                State::AttributeValueDoubleQuoted => todo!("AttributeValueDoubleQuoted"),
                State::AttributeValueSingleQuoted => todo!("AttributeValueSingleQuoted"),
                State::AttributeValueUnquoted => todo!("AttributeValueUnquoted"),
                State::AfterAttributeValueQuoted => todo!("AfterAttributeValueQuoted"),
                State::SelfClosingStartTag => todo!("SelfClosingStartTag"),
                State::BogusComment => todo!("BogusComment"),
                State::MarkupDeclarationOpen => todo!("MarkupDeclarationOpen"),
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
                State::Doctype => todo!("Doctype"),
                State::BeforeDoctypeName => todo!("BeforeDoctypeName"),
                State::DoctypeName => todo!("DoctypeName"),
                State::AfterDoctypeName => todo!("AfterDoctypeName"),
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
        self.tokens.clone()
    }

    fn current_input_character(&self) -> Option<char> {
        self.html.chars().nth(self.insertion_point)
    }

    fn next_input_character(&mut self) -> Option<char> {
        self.html.chars().nth(self.insertion_point + 1)
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

    fn emit_token(&mut self, token: Token) {
        self.tokens.push(token);
    }

    fn emit_current_token(&mut self) {
        if let Some(token) = self.current_token.take() {
            self.emit_token(token);
            self.current_token = None;
        }
    }
}
