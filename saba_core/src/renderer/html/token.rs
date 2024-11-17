use alloc::string::String;
use alloc::vec::Vec;
use crate::renderer::html::attribute::Attribute;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HtmlTokenizer
{
    state: State,
    pos: usize,
    reconsume: bool,
    latest_token: Option<HtmlToken>,
    input: Vec<char>,
    buf: String,
}

impl HtmlTokenizer
{
    pub fn new(html: String) -> Self
    {
        Self
        {
            state: State::Data,
            pos: 0,
            reconsume: false,
            latest_token: None,
            input: html.chars().collect(),
            buf: String::new(),
        }
    }

    fn consume_next_input(&mut self) -> char
    {
        let c = self.input[self.pos];
        self.pos += 1;
        c
    }

    fn crate_tag(&mut self, start_tag_token: bool)
    {
        if start_tag_token
        {
            self.latest_token = Some(HtmlToken::StartTag {
                tag: String::new(),
                self_closing: false,
                attributes: Vec::new(),
            });
        } else {
            self.latest_token = Some(HtmlToken::EndTag {
                tag: String::new(),
            });
        }
    }

    fn reconsume_input(&mut self) -> char
    {
        self.reconsume = false;
        self.input[self.pos - 1]
    }

    fn append_tag_name(&mut self, c: char)
    {
        assert!(self.latest_token.is_some());

        if let Some(t) = self.latest_token.as_mut()
        {
            match t
            {
                HtmlToken::StartTag {
                    ref mut tag,
                    self_closing: _,
                    attributes: _,
                }
                | HtmlToken::EndTag { ref mut tag } => tag.push(c),
                _ => {
                    panic!("`latest_token` should be either StartTag or EndTag");
                }
            }
        }
    }

    fn take_latest_token(&mut self) -> Option<HtmlToken>
    {
        assert!(self.latest_token.is_some());

        let t = self.latest_token.as_ref().cloned();
        self.latest_token = None;
        assert!(self.latest_token.is_none());

        t
    }

    fn start_new_attribute(&mut self)
    {
        assert!(self.latest_token.is_some());

        if let Some(t) = self.latest_token.as_mut()
        {
            match t
            {
                HtmlToken::StartTag {
                    tag: _,
                    self_closing: _,
                    ref mut attributes,
                } => {
                    attributes.push(Attribute::new());
                }
                _ => {
                    panic!("`latest_token` should be either StartTag");
                }
            }
        }
    }

    fn append_attribute(&mut self, c: char, is_name: bool)
    {
        assert!(self.latest_token.is_some());

        if let Some(t) = self.latest_token.as_mut()
        {
            match t
            {
                HtmlToken::StartTag
                {
                    tag: _,
                    self_closing: _,
                    ref mut attributes,
                } =>
                    {
                        let len = attributes.len();
                        assert!(len > 0);

                        attributes[len - 1].add_char(c, is_name);
                    }
                _ =>
                    {
                        panic!("`latest_token` should be either StartTag");
                    }
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HtmlToken
{
    StartTag
    {
        tag: String,
        self_closing: bool,
        attributes: Vec<Attribute>,
    },

    EndTag
    {
        tag: String,
    },

    Char(char),
    Eof,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum State
{
    Data,
    TagOpen,
    EndTagOpen,
    TagName,
    BeforeAttributeName,
    AttributeName,
    AfterAttributeName,
    BeforeAttributeValue,
    AttributeValueDoubleQuoted,
    AttributeValueSingleQuoted,
    AttributeValueUnquoted,
    AfterAttributeValueQuoted,
    SelfClosingStartTag,
    ScriptData,
    ScriptDataLessThanSign,
    ScriptDataEndTagOpen,
    ScriptDataEndTagName,
    TemporaryBuffer,
}

impl Iterator for HtmlTokenizer
{
    type Item = HtmlToken;

    fn next(&mut self) -> Option<Self::Item>
    {
        loop {
            let c = match self.reconsume
            {
                true => self.reconsume_input(),
                false => self.consume_next_input(),
            };

            match self.state
            {
                State::Data =>
                    {
                        if c == '<'
                        {
                            self.state = State::TagOpen;
                            continue;
                        }

                        if self.is_eof()
                        {
                            return Some(HtmlToken::Eof);
                        }

                        return Some(HtmlToken::Char(c));
                    }
                State::TagOpen => {
                    if c == '/'
                    {
                        self.state = State::EndTagOpen;
                        continue;
                    }

                    if c.is_ascii_alphabetic()
                    {
                        self.reconsume = true;
                        self.state = State::TagName;
                        self.crate_tag(true);
                        continue;
                    }

                    if self.is_eof()
                    {
                        return Some(HtmlToken::Eof);
                    }

                    self.reconsume = true;
                    self.state = State::Data;
                }
                State::EndTagOpen =>
                    {
                        if self.is_eof()
                        {
                            return Some(HtmlToken::Eof);
                        }

                        if c.is_ascii_alphabetic()
                        {
                            self.reconsume = true;
                            self.state = State::TagName;
                            self.crate_tag(false);
                            continue;
                        }
                    }
                State::TagName =>
                    {
                        if c == ' '
                        {
                            self.state = State::BeforeAttributeName;
                            continue;
                        }

                        if c == '/'
                        {
                            self.state = State::SelfClosingStartTag;
                            continue;
                        }

                        if c == '>'
                        {
                            self.state = State::Data;
                            return self.take_last_token();
                        }

                        if c.is_ascii_uppercase()
                        {
                            self.append_tag_name(c.to_ascii_lowercase());
                            continue;
                        }

                        if self.eof()
                        {
                            return Some(HtmlToken::Eof);
                        }

                        self.append_tag_name(c);
                    }
                State::BeforeAttributeName =>
                    {
                        if c == '/' || c == '>' || self.is_eof()
                        {
                            self.reconsume = true;
                            self.state = State::AfterAttributeName;
                            continue;
                        }

                        self.reconsume = true;
                        self.state = State::AttributeName;
                        self.start_new_attribute();
                    }
                State::AttributeName =>
                    {
                        if c == ' ' || c == '/' || c == '>' || self.is_eof()
                        {
                            self.reconsume = true;
                            self.state = State::AfterAttributeName;
                            continue;
                        }

                        if c == '='
                        {
                            self.state = State::BeforeAttributeValue;
                            continue;
                        }

                        if c.is_ascii_uppercase()
                        {
                            self.append_attribute(c.to_ascii_lowercase(),
                                                  /* is_name = */ true);

                            continue;
                        }

                        self.append_attribute(c, /* is_name = */ true);
                    }
                State::AfterAttributeName => {}
                State::BeforeAttributeValue => {}
                State::AttributeValueDoubleQuoted => {}
                State::AttributeValueSingleQuoted => {}
                State::AttributeValueUnquoted => {}
                State::AfterAttributeValueQuoted => {}
                State::SelfClosingStartTag => {}
                State::ScriptData => {}
                State::ScriptDataLessThanSign => {}
                State::ScriptDataEndTagOpen => {}
                State::ScriptDataEndTagName => {}
                State::TemporaryBuffer => {}
            }
        }
    }
}