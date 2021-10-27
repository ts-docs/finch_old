
use std::iter::{Peekable};
use std::ops::Range;
use crate::error::*;

type Data<'a> = Peekable<std::str::CharIndices<'a>>;
type FinchResult<R> = Result<R, FinchError>;

static COMPARE_PREC: i8 = 5;
static AND_PREC: i8 = 10;
static OR_PREC: i8 = 7;

pub enum BinaryOps {
    Compare(ExpressionKind, ExpressionKind), 
    Not(ExpressionKind, ExpressionKind),
    Gt(ExpressionKind, ExpressionKind),
    Lt(ExpressionKind, ExpressionKind),
    Gte(ExpressionKind, ExpressionKind),
    Lte(ExpressionKind, ExpressionKind),
    And(ExpressionKind, ExpressionKind),
    Or(ExpressionKind, ExpressionKind)
}

pub enum UnaryOps {
    Not(ExpressionKind)
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
    Unary(Box<UnaryOps>),
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

    pub fn parse(str: &'a str) -> FinchResult<(Vec<char>, SubText)> {
        let mut p = Self {
            data: str.char_indices().peekable()
        };
        let res = p.parse_text();
        Ok((vec![], res?))
    }

    pub fn parse_text(&mut self) -> FinchResult<SubText> {
        let mut current = self.data.next();
        let first_ind = current.ok_or_else(FinchError::none)?.0;
        let mut last_ind: usize = 0;
        let mut templates: Vec<Template> = vec![];
        while let Some(ch) = current {
            last_ind = ch.0;
            if ch.1 == '{' && self.is_next('{') {
                self.data.next();
                if self.is_next('#') {
                    self.data.next();
                    let temp_kind = self.parse_block()?;
                    templates.push(Template { pos: ch.0..self.data.peek().ok_or_else(FinchError::none)?.0, kind: temp_kind });
                } else {
                    let temp_kind = self.parse_full_expression()?;
                    self.skip_token('}')?;
                    self.skip_token('}')?;
                    // Plus 2 because of the }}
                    templates.push(Template { pos: ch.0..(temp_kind.0 + 2), kind: TemplateKind::Expression(temp_kind.1) });
                }
            }
            current = self.data.next();
        }
        Ok(SubText {
            pos: first_ind..last_ind,
            templates
        })
    }

    pub fn parse_block(&mut self) -> FinchResult<TemplateKind> {
        Ok(TemplateKind::Expression(ExpressionKind::String(String::from("TEST_BLOCK"))))
    }

    pub fn parse_expression(&mut self) -> FinchResult<ExpressionKind> {
        let current = self.data.next().ok_or_else(FinchError::none)?;
        match current.1 {
            '"' => Ok(ExpressionKind::String(self.parse_string()?)),
            '0'..='9' | '-' => Ok(ExpressionKind::Number(self.parse_number(current.1)?)),
            'a'..='z' | 'A'..='Z' | '_' | '$' => Ok(self.parse_possible_var(current.1)?),
            ' ' => {
                self.skip_while(' ');
                self.parse_expression()
            },
            '(' => {
                let exp = self.parse_full_expression()?;
                self.skip_token(')')?;
                Ok(exp.1)
            },
            '!' => {
                let exp = self.parse_expression()?;
                Ok(ExpressionKind::Unary(Box::new(UnaryOps::Not(exp))))
            }
            _ => Err(FinchError(FinchErrorKind::Unexpected(current.1)))
        }
    }

    pub fn parse_full_expression(&mut self) -> FinchResult<(usize, ExpressionKind)> {
        let exp = self.parse_expression()?;
        self.parse_possibly_binary(exp, -1)
    }

    fn parse_possibly_binary(&mut self, res: ExpressionKind, prec: i8) -> FinchResult<(usize, ExpressionKind)> {
        let followup = self.data.peek().ok_or_else(FinchError::none)?;
        let followup_end = followup.0;
        match followup.1 {
            '=' => { // ==
                self.data.next();
                self.skip_token('=')?;
                let right = self.parse_expression()?;
                self.parse_possibly_binary(ExpressionKind::Binary(Box::new(BinaryOps::Compare(res, right))), COMPARE_PREC)
            },
            '!' => { // !=
                self.data.next();
                self.skip_token('=')?;
                let right = self.parse_expression()?;
                self.parse_possibly_binary(ExpressionKind::Binary(Box::new(BinaryOps::Not(res, right))), COMPARE_PREC)
            }
            '&' => { // &&
                self.data.next();
                self.skip_token('&')?;
                if AND_PREC > prec {
                    let right = self.parse_full_expression()?.1;
                    self.parse_possibly_binary(ExpressionKind::Binary(Box::new(BinaryOps::And(res, right))), AND_PREC)
                } else {
                    let right = self.parse_expression()?;
                    self.parse_possibly_binary(ExpressionKind::Binary(Box::new(BinaryOps::And(res, right))), AND_PREC)
                }
            }
            '|' => { // ||
                self.data.next();
                self.skip_token('|')?;
                if OR_PREC > prec {
                    let right = self.parse_full_expression()?.1;
                    self.parse_possibly_binary(ExpressionKind::Binary(Box::new(BinaryOps::Or(res, right))), OR_PREC)
                } else {
                    let right = self.parse_expression()?;
                    self.parse_possibly_binary(ExpressionKind::Binary(Box::new(BinaryOps::Or(res, right))), OR_PREC)
                }
            },
            '(' => { // function_call();
                self.data.next();
                let mut params: Vec<ExpressionKind> = vec![];
                while let Some(ch) = self.data.peek() {
                    match ch.1 {
                        ' ' | ',' => {
                            self.data.next();
                            continue;
                        },
                        ')' => {
                            self.data.next();
                            return self.parse_possibly_binary(ExpressionKind::Call {
                                var: Box::from(res),
                                params
                            }, 1);
                        }
                        _ => {
                            let exp = self.parse_full_expression()?;
                            params.push(exp.1);
                        }
                    }
                }
                Err(FinchError::none())
            }
            ' ' => {
                self.skip_while(' ');
                self.parse_possibly_binary(res, prec)
            }
            _ => Ok((followup_end, res))
        }
    }

    fn parse_possible_var(&mut self, start: char) -> FinchResult<ExpressionKind> {
        let mut res = String::from(start);
        let mut vault: Vec<String> = vec![];
        while let Some(ch) = self.data.peek() {
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
                    if vault.is_empty() {
                        if res.is_empty() {
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
        }
        Err(FinchError::none())
    }

    fn parse_string(&mut self) -> FinchResult<String> {
        let mut res = String::new();
        while let Some(ch) = self.data.next() {
            match ch.1 {
                '"' => {
                    return Ok(res);
                },
                '\\' => res.push(self.data.next().ok_or(FinchError(FinchErrorKind::Expected('"')))?.1),
                _ => res.push(ch.1)
            }
        }
        Err(FinchError(FinchErrorKind::Expected('"')))
    }

    fn parse_number(&mut self, last: char) -> FinchResult<f32> {
        let mut res = String::from(last);
        let mut has_floating_point = false;
        while let Some(ch) = self.data.peek() {
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
                _ => return res.parse::<f32>().map_err(|_| FinchError(FinchErrorKind::InvalidNumber))
            }
        }
        Err(FinchError::none())
    }

    fn is_next(&mut self, ch: char) -> bool {
        if let Some(thing) = self.data.peek() {
            thing.1 == ch
        } else {
            false
        }
    }

    fn skip_token(&mut self, ch: char) -> FinchResult<()> {
        if let Some(next) = self.data.next() {
            if next.1 != ch {
                Err(FinchError(FinchErrorKind::ExpectedFound(ch, next.1)))
            } else {
                Ok(())
            }
        } else {
            Err(FinchError(FinchErrorKind::Expected(ch)))
        }
    }

    fn skip_while(&mut self, ch: char) {
        while let Some(character) = self.data.peek() {
            if character.1 == ch {
                self.data.next();
            } else {
                break;
            }
        }
    }

}