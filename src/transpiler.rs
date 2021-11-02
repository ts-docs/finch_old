use crate::parser::*;
use neon::prelude::{Handle, JsFunction, FunctionContext};
use std::collections::HashMap;

pub struct Transpiler {}

pub struct TranspilerContext<'a, 'b> {
    pub transpiler: &'a Transpiler,
    pub cx: &'a mut FunctionContext<'b>,
    pub original: &'a str
}


impl Transpiler {

    pub fn new() -> Self {
        Self {}
    }


}

pub trait Transpilable {
    fn transpile(self, ctx: &mut TranspilerContext) -> String;
}

impl Transpilable for ExpressionKind {

    fn transpile(self, ctx: &mut TranspilerContext) -> String {
        match self {
            ExpressionKind::String(str) => str,
            ExpressionKind::Number(num) => num.to_string(),
            _ => String::new()
        }
    }
}