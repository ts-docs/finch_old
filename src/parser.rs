
use std::iter::{Peekable};
use std::ops::Range;
use crate::error::*;

type Data<'a> = Peekable<std::str::CharIndices<'a>>;
type FinchResult<R> = Result<R, FinchError>;

pub enum BinaryOps {
    Compare(ExpressionKind, ExpressionKind), 
    Not(ExpressionKind, ExpressionKind),
    Gt(ExpressionKind, ExpressionKind),
    Lt(ExpressionKind, ExpressionKind),
    Gte(ExpressionKind, ExpressionKind),
    Lte(ExpressionKind, ExpressionKind)
}

pub enum ExpressionKind {
    Var(String),
    VarDot(Vec<String>),
    Number(f32),
    String(String),
    Bool(bool),
    Undefined,
    Null,
    Binary(Box<BinaryOps>),
    Call {
        var: Box<ExpressionKind>,
        params: Vec<ExpressionKind>
    }
}


pub enum TemplateKind {
    Expression(ExpressionKind),
    Block {
        name: String,
        params: Vec<ExpressionKind>,
        block: SubText
    }
}

pub struct Template {
    pub pos: Range<usize>,
    pub kind: TemplateKind
}

pub struct SubText {
    pub pos: Range<usize>,
    pub templates: Vec<Template>
}

pub struct Parser<'a> {
    data: Data<'a>
}

impl<'a> Parser<'a> {

    pub fn parse(str: &'a String) -> FinchResult<(Vec<char>, SubText)> {
        let mut p = Self {
            data: str.char_indices().peekable()
        };
        let res = p.parse_text();
        Ok((vec![], res?))
    }

    pub fn parse_text(&mut self) -> FinchResult<SubText> {
        let mut current = self.data.next();
        let first_ind = current.ok_or(FinchError::none())?.0;
        let mut last_ind: usize = 0;
        let mut templates: Vec<Template> = vec![];
        while let Some(ch) = current {
            last_ind = ch.0;
            if ch.1 == '{' && self.is_next('{') {
                self.data.next();
                if self.is_next('#') {
                    self.data.next();
                    let temp_kind = self.parse_block()?;
                    templates.push(Template { pos: first_ind..self.data.peek().ok_or(FinchError::none())?.0, kind: temp_kind });
                } else {
                    let temp_kind = self.parse_expression()?;
                    self.skip_token('}')?;
                    templates.push(Template { pos: ch.0..(temp_kind.0 + 1), kind: TemplateKind::Expression(temp_kind.1) });
                }
            } else {
                current = self.data.next();
            }
        }
        Ok(SubText {
            pos: first_ind..last_ind,
            templates
        })
    }

    pub fn parse_block(&mut self) -> FinchResult<TemplateKind> {
        Ok(TemplateKind::Expression(ExpressionKind::String(String::from("TEST_BLOCK"))))
    }

    pub fn parse_expression(&mut self) -> FinchResult<(usize, ExpressionKind)> {
        let current = self.data.next().ok_or(FinchError::none())?;
        let res = match current.1 {
            '"' => ExpressionKind::String(self.parse_string()?),
            '1'..='9' => ExpressionKind::Number(self.parse_number(current.1)?),
            'a'..='z' | 'A'..='Z' | '_' | '$' => self.parse_possible_var(current.1)?,
            ' ' => {
                self.skip_while(' ');
                self.parse_expression()?.1
            },
            '(' => {
                let exp = self.parse_expression()?.1;
                self.skip_token(')')?;
                exp
            }
            _ => return Err(FinchError(FinchErrorKind::Unexpected(current.1)))
        };
        self.parse_followup(res)
    }

    fn parse_followup(&mut self, res: ExpressionKind) -> FinchResult<(usize, ExpressionKind)> {
        let followup = self.data.peek().ok_or(FinchError::none())?;
        match followup.1 {
            '=' => {
                self.data.next();
                self.skip_token('=')?;
                let right = self.parse_expression()?;
                Ok((right.0, ExpressionKind::Binary(Box::new(BinaryOps::Compare(res, right.1)))))
            },
            ' ' => {
                self.skip_while(' ');
                self.parse_followup(res)
            }
            _ => Ok((followup.0, res))
        }
    }

    fn parse_possible_var(&mut self, start: char) -> FinchResult<ExpressionKind> {
        let mut res = String::from(start);
        let mut vault: Vec<String> = vec![];
        loop {
            if let Some(ch) = self.data.peek() {
                match ch.1 {
                    'a'..='z' | 'A'..='Z' | '_' | '$' | '0'..='9' => {
                        res.push(ch.1);
                        self.data.next();
                    }
                    '.' => {
                        vault.push(res);
                        res = String::new();
                        self.data.next();
                    }
                    _ => {
                        if vault.len() != 0 {
                            if res.len() == 0 {
                                return Err(FinchError(FinchErrorKind::MissingPropName));
                            }
                            vault.push(res);
                            return Ok(ExpressionKind::VarDot(vault));
                        }
                        return Ok(match res.as_str() {
                            "true" => ExpressionKind::Bool(true),
                            "false" => ExpressionKind::Bool(false),
                            "undefined" => ExpressionKind::Undefined,
                            "null" => ExpressionKind::Null,
                            _ => ExpressionKind::Var(res)
                        })
                    }
                }
            } else {
                return Err(FinchError::none());
            }
        }
    }

    fn parse_string(&mut self) -> FinchResult<String> {
        let mut res = String::new();
        loop {
            if let Some(ch) = self.data.next() {
                match ch.1 {
                    '"' => {
                        return Ok(res);
                    },
                    '\\' => res.push(self.data.next().ok_or(FinchError(FinchErrorKind::Expected('"')))?.1),
                    _ => res.push(ch.1)
                }
            } else {
                return Err(FinchError(FinchErrorKind::Expected('"')));
            }
        }
    }

    fn parse_number(&mut self, last: char) -> FinchResult<f32> {
        let mut res = String::from(last);
        let mut has_floating_point = false;
        loop {
            if let Some(ch) = self.data.peek() {
                match ch.1 {
                    '0'..='9' => {
                        res.push(ch.1);
                        self.data.next();
                    },
                    '_' => {
                        self.data.next();
                        continue;
                    },
                    '.' if !has_floating_point => {
                        has_floating_point = true;
                        res.push('.');
                        self.data.next();
                    },
                    _ => return Ok(res.parse::<f32>().map_err(|_| FinchError(FinchErrorKind::InvalidNumber))?)
                }
            } else {
                return Err(FinchError::none())
            }
        }
    }

    fn is_next(&mut self, ch: char) -> bool {
        let thing = self.data.peek();
        if thing.is_none() { false }
        else { thing.unwrap().1 == ch }
    }

    fn skip_token(&mut self, ch: char) -> FinchResult<()> {
        if let Some(next) = self.data.next() {
            if next.1 != ch {
                return Err(FinchError(FinchErrorKind::ExpectedFound(ch, next.1)))
            } else {
                Ok(())
            }
        } else {
            Err(FinchError(FinchErrorKind::Expected(ch)))
        }
    }

    fn skip_while(&mut self, ch: char) -> usize {
        let mut character = self.data.peek();
        let mut count: usize = 0;
        loop {
            if let Some(unwrapped) = character {
                if unwrapped.1 == ch {
                    self.data.next();
                    character = self.data.peek();
                    count += 1;
                }
                else {
                    break;
                }
            }
        }
        count
    }

}