use crate::renderer::css::token::CssTokenizer;
use core::iter::Peekable;
use alloc::vec::Vec;
use crate::alloc::string::ToString;
use crate::alloc::string::String;
use crate::renderer::css::token::CssToken;

pub type ComponentValue = CssToken;

#[derive(Debug, Clone)]
pub struct CssParser {
    t: Peekable<CssTokenizer>,
}

impl CssParser
{
    pub fn new(t: CssTokenizer) -> Self
    {
        Self {
            t: t.peekable(),
        }
    }

    pub fn parse_stylesheet(&mut self) -> Stylesheet
    {
        let mut sheet = Stylesheet::new();

        sheet.set_rules(self.parse_rules());
        sheet
    }

    fn consume_list_of_rules(&mut self) -> Vec<QualifiedRule>
    {
        let mut rules = Vec::new();

        loop
        {
            let token = match self.t.peek()
            {
                Some(t) => t,
                None => return rules,
            };

            match token
            {
                CssToken::AtKeyword(_keyword) =>
                    {
                        let _rule = self.consume_at_rule();
                    }
                _ =>
                    {
                        let rule = self.consume_qualified_rule();
                        match rule
                        {
                            Some(r) => rules.push(r),
                            None => return rules,
                        }
                    }
            }
        }
    }

}

#[derive(Debug, Clone, PartialEq)]
pub struct Stylesheet {
    pub rules: Vec<QualifiedRule>,
}

impl Stylesheet
{
    pub fn new() -> Self
    {
        Self {
            rules: Vec::new(),
        }
    }

    pub fn set_rules(&mut self, rules: Vec<QualifiedRule>)
    {
        self.rules = rules;
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct QualifiedRule
{
    pub selector: Selector,
    pub declarations: Vec<Declaration>,
}

impl QualifiedRule
{
    pub fn new() -> Self
    {
        Self {
            selector: Selector::TypeSelector("".to_string()),
            declarations: Vec::new(),
        }
    }

    pub fn set_selector(&mut self, selector: Selector)
    {
        self.selector = selector;
    }

    pub fn set_declarations(&mut self, declarations: Vec<Declaration>)
    {
        self.declarations = declarations;
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Selector
{
    TypeSelector(String),
    ClassSelector(String),
    IdSelector(String),
    UnknownSelector,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Declaration
{
    pub property: String,
    pub value: ComponentValue,
}

impl Declaration
{
    pub fn new() -> Self
    {
        Self
        {
            property: String::new(),
            value: ComponentValue::Ident(String::new()),
        }
    }

    pub fn set_property(&mut self, property: String)
    {
        self.property = property;
    }

    pub fn set_value(&mut self, value: ComponentValue)
    {
        self.value = value;
    }
}

