
use std::iter::{Peekable};
use std::ops::Range;
use crate::error::*;

type Data<'a> = Peekable<std::str::CharIndices<'a>>;
pub type FinchResult<R> = Result<R, FinchError>;

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
    Not(ExpressionKind),
    ShortCircuit(ExpressionKind)
}

pub enum ExpressionKind {
    Var(String),
    VarDot(Vec<String>),
    Number(f64),
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

pub struct FnBlock {
    pub name: String,
    pub params: Vec<ExpressionKind>,
    pub block: Option<SubText>,
    pub chain: Option<Box<FnBlock>>
}

pub enum TemplateKind {
    Expression(ExpressionKind),
    Block(FnBlock)
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

    pub fn parse(str: &'a str) -> FinchResult<SubText> {
        let mut p = Self {
            data: str.char_indices().peekable()
        };
        let mut current = p.data.next();
        let first_ind = current.ok_or(FinchError::None)?.0;
        let mut last_ind: usize = 0;
        let mut templates: Vec<Template> = vec![];
        while let Some(ch) = current {
            if ch.1 == '{' && p.is_next('{') {
                p.data.next();
                if p.is_next('#') {
                    p.data.next();
                    let temp_kind = p.parse_block()?;
                    // Plus 1 because of #
                    templates.push(Template { pos: ch.0..(temp_kind.0 + 1), kind: TemplateKind::Block(temp_kind.1) });
                } else {
                    let temp_kind = p.parse_full_expression()?;
                    p.skip_token('}')?;
                    p.skip_token('}')?;
                    // Plus 2 because of the }}
                    templates.push(Template { pos: ch.0..(temp_kind.0 + 2), kind: TemplateKind::Expression(temp_kind.1) });
                }
            }
            current = p.data.next();
            last_ind = ch.0;
        }
        Ok(SubText {
            pos: first_ind..(last_ind + 1),
            templates
        })
    }

    // Parses text INSIDE a function block
    fn parse_text(&mut self) -> FinchResult<(usize, SubText, Option<Box<FnBlock>>)> {
        let mut templates: Vec<Template> = vec![];
        let start = self.data.peek().ok_or(FinchError::None)?.0;
        while let Some(ch) = self.data.next() {
            if ch.1 == '{' && self.is_next('{') {
                self.data.next();
                let next = self.data.peek().ok_or(FinchError::None)?;
                let next_start = next.0;
                match next.1 {
                    '#' => {
                        self.data.next();
                        let temp_kind = self.parse_block()?;
                        templates.push(Template { pos: ch.0..temp_kind.0, kind: TemplateKind::Block(temp_kind.1) });
                    },
                    '/' => {
                        self.data.next();
                        let end: usize;
                        let chain = if self.is_next('#') {
                            self.data.next();
                            let bl = self.parse_block()?;
                            end = bl.0;
                            Some(Box::from(bl.1))
                        } else { 
                            self.skip_token('}')?;
                            self.skip_token('}')?;
                            end = next_start + 2;
                            None
                        };
                        return Ok((
                            end,
                            SubText { pos: start..(next_start - 1), templates },
                            chain
                        ))
                    },
                    _ => {
                        let temp_kind = self.parse_full_expression()?;
                        self.skip_token('}')?;
                        self.skip_token('}')?;
                        // Plus 2 because of the }}
                        templates.push(Template { pos: ch.0..(temp_kind.0 + 2), kind: TemplateKind::Expression(temp_kind.1) });
                    }
                }
            }
        }
        Err(FinchError::None)
    }

    pub fn parse_block(&mut self) -> FinchResult<(usize, FnBlock)> {
        let fn_name = self.parse_var()?;
        let mut params: Vec<ExpressionKind> = vec![];
        while let Some(ch) = self.data.peek() {
            match ch.1 {
                ' ' | ',' => {
                    self.data.next();
                    continue;
                },
                '/' => {
                    let ch_end = ch.0;
                    self.data.next();
                    self.skip_token('}')?;
                    self.skip_token('}')?;
                    return Ok((ch_end, FnBlock {
                        name: fn_name,
                        params, 
                        block: None,
                        chain: None
                    }))
                }
                '}' => {
                    self.data.next();
                    self.skip_token('}')?;
                    let end = self.parse_text()?;
                    return Ok((end.0, FnBlock {
                        name: fn_name,
                        params,
                        block: Some(end.1),
                        chain: end.2
                    }))
                }
                _ => params.push(self.parse_full_expression()?.1),
            }
        }
        Err(FinchError::None)
    }

    pub fn parse_expression(&mut self) -> FinchResult<ExpressionKind> {
        let current = self.data.peek().ok_or(FinchError::None)?;
        let thing = match current.1 {
            '"' => ExpressionKind::String(self.parse_string()?),
            '0'..='9' | '-' => ExpressionKind::Number(self.parse_number()?),
            'a'..='z' | 'A'..='Z' | '_' | '$' => self.parse_possible_var()?,
            ' ' => {
                self.skip_while(' ');
                self.parse_expression()?
            },
            '(' => {
                self.data.next();
                let exp = self.parse_full_expression()?;
                self.skip_token(')')?;
                exp.1
            },
            '!' => {
                self.data.next();
                ExpressionKind::Unary(Box::new(UnaryOps::Not(self.parse_expression()?)))
            },
            _ => return Err(FinchError::Unexpected(current.1))
        };
        let next = self.data.peek().ok_or(FinchError::None)?;
        match next.1 {
            '?' => {
                self.data.next();
                Ok(ExpressionKind::Unary(Box::new(UnaryOps::ShortCircuit(thing))))
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
                            return Ok(ExpressionKind::Call {
                                var: Box::from(thing),
                                params
                            });
                        }
                        _ => params.push(self.parse_full_expression()?.1)
                    }
                }
                Err(FinchError::None)
            },
            _ => Ok(thing)
        }
    }

    pub fn parse_full_expression(&mut self) -> FinchResult<(usize, ExpressionKind)> {
        let exp = self.parse_expression()?;
        self.parse_possibly_binary(exp, -1)
    }

    fn parse_possibly_binary(&mut self, res: ExpressionKind, prec: i8) -> FinchResult<(usize, ExpressionKind)> {
        let followup = self.data.peek().ok_or(FinchError::None)?;
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
            '>' => { // >, >=
                self.data.next();
                if self.is_next('=') {
                    self.data.next();
                    let right = self.parse_expression()?;
                    self.parse_possibly_binary(ExpressionKind::Binary(Box::new(BinaryOps::Gte(res, right))), COMPARE_PREC)
                } else {
                    let right = self.parse_expression()?;
                    self.parse_possibly_binary(ExpressionKind::Binary(Box::new(BinaryOps::Gt(res, right))), COMPARE_PREC)
                }
            },
            '<' => { // >, >=
                self.data.next();
                if self.is_next('=') {
                    self.data.next();
                    let right = self.parse_expression()?;
                    self.parse_possibly_binary(ExpressionKind::Binary(Box::new(BinaryOps::Lte(res, right))), COMPARE_PREC)
                } else {
                    let right = self.parse_expression()?;
                    self.parse_possibly_binary(ExpressionKind::Binary(Box::new(BinaryOps::Lt(res, right))), COMPARE_PREC)
                }
            },
            ' ' => {
                self.skip_while(' ');
                self.parse_possibly_binary(res, prec)
            }
            _ => Ok((followup_end, res))
        }
    }

    fn parse_possible_var(&mut self) -> FinchResult<ExpressionKind> {
        let mut res = String::new();
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
                    if !vault.is_empty() {
                        if res.is_empty() {
                            return Err(FinchError::MissingPropName);
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
        Err(FinchError::None)
    }

    // Parses ONLY a variable and returns it's contents
    fn parse_var(&mut self) -> FinchResult<String> {
        let mut res = String::new();
        while let Some(ch) = self.data.peek() {
            match ch.1 {
                'a'..='z' | 'A'..='Z' | '_' | '$' | '0'..='9' => {
                    res.push(ch.1);
                    self.data.next();
                }
                _ => {
                    return Ok(res);
                }
            }
        }
        Err(FinchError::None)
    }

    fn parse_string(&mut self) -> FinchResult<String> {
        self.data.next(); // Skip "
        let mut res = String::new();
        while let Some(ch) = self.data.next() {
            match ch.1 {
                '"' => {
                    return Ok(res);
                },
                '\\' => res.push(self.data.next().ok_or(FinchError::Expected('"'))?.1),
                _ => res.push(ch.1)
            }
        }
        Err(FinchError::Expected('"'))
    }

    fn parse_number(&mut self) -> FinchResult<f64> {
        let mut res = String::new();
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
                _ => return res.parse::<f64>().map_err(|_| FinchError::InvalidNumber)
            }
        }
        Err(FinchError::None)
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
                Err(FinchError::ExpectedFound(ch, next.1))
            } else {
                Ok(())
            }
        } else {
            Err(FinchError::Expected(ch))
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