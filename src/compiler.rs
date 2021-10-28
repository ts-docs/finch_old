
use crate::parser::*;
use crate::error::FinchError;
use crate::convert::*;
use neon::types::{JsObject, JsString};
use neon::handle::Handle;
use neon::object::Object;
use neon::context::{FunctionContext};
use std::collections::HashMap;

pub struct Compiler {
    pub templates: HashMap<String, (String, SubText)>,
}

pub struct CompilerContext<'a, 'b> {
    cx: &'a mut FunctionContext<'b>,
    locals: HashMap<String, RawValue>,
    data: &'a Handle<'a, JsObject>,
    original: &'a str
}

impl Compiler {

    pub fn new() -> Self {
        Self { 
            templates: HashMap::new(),
         }
    }

    pub fn add_template(&mut self, name: &str, text: &str) -> FinchResult<()> {
        let parsed = Parser::parse(text)?;
        self.templates.insert(name.to_string(), (text.to_string(), parsed));
        Ok(())
    }

    pub fn compile(&self, cx: &mut FunctionContext) -> FinchResult<String> {
        let name = cx.argument::<JsString>(0).map_err(|_| FinchError::InvalidArg(0))?.value(cx);
        let data= cx.argument::<JsObject>(1).map_err(|_| FinchError::InvalidArg(1))?;
        let (og, temp) = self.templates.get(&name).ok_or(FinchError::TemplateNotExist(name))?;
        temp.compile(&mut CompilerContext {
            cx,
            locals: HashMap::new(),
            data: &data,
            original: &og
        })
    }

}

pub trait Compilable<T> {
    fn compile(&self, ctx: &mut CompilerContext) -> FinchResult<T>;
}

impl Compilable<String> for SubText {

    fn compile(&self, ctx: &mut CompilerContext) -> FinchResult<String> {
        let mut res = String::new();
        let mut last_temp_end = 0;
        for temp in &self.templates {
            let temp_str = match &temp.kind {
                TemplateKind::Expression(exp) => exp.compile(ctx)?.as_string(),
                TemplateKind::Block(_) => String::new()
            };
            if last_temp_end < temp.pos.start {
                res += &ctx.original[last_temp_end..temp.pos.start];
            }
            last_temp_end = temp.pos.end;
            res += &temp_str;
        }
        res += &ctx.original[last_temp_end..ctx.original.len()];
        Ok(res)
    }
}

impl Compilable<RawValue> for ExpressionKind {

    fn compile(&self, ctx: &mut CompilerContext) -> FinchResult<RawValue> {
        match self {
            ExpressionKind::String(val) => Ok(RawValue::String(val.to_string())),
            ExpressionKind::Bool(val) => Ok(RawValue::Boolean(*val)),
            ExpressionKind::Number(val) => Ok(RawValue::Number(*val)),
            ExpressionKind::Null => Ok(RawValue::Null),
            ExpressionKind::Undefined => Ok(RawValue::Undefined),
            ExpressionKind::Var(val) => {
                if let Some(thing) = ctx.locals.get(val) {
                    return Ok(thing.clone());
                }
                let dat = ctx.data.get(ctx.cx, val.as_str()).map_err(|_| FinchError::PropNotExist(val.to_string()))?.raw(ctx.cx);
                ctx.locals.insert(val.to_string(), dat.clone());
                Ok(dat)
            },
            ExpressionKind::VarDot(path) => {
                let joined = path.join(".");
                if let Some(thing) = ctx.locals.get(&joined) {
                    return Ok(thing.clone());
                }
                let first = &path[0];
                let mut dat = ctx.data.get(ctx.cx, first.as_str()).map_err(|_| FinchError::PropNotExist(first.to_string()))?;
                for ind in 1..path.len() {
                    dat = dat.downcast::<JsObject, _>(ctx.cx).map_err(|_| FinchError::ExpectedObject)?.get(ctx.cx, path[ind].as_str()).map_err(|_| FinchError::ExpectedObject)?;
                };
                let raw_thing = dat.raw(ctx.cx);
                ctx.locals.insert(joined, raw_thing.clone());
                Ok(raw_thing)
            }
            ExpressionKind::Unary(exp) => {
                match &**exp {
                    UnaryOps::Not(exp) => {
                        let compiled = exp.compile(ctx)?;
                        Ok(RawValue::Boolean(compiled.is_falsey()))
                    }
                }
            },
            ExpressionKind::Binary(exp) => {
                match &**exp {
                    BinaryOps::Compare(left, right) => {
                        Ok(RawValue::Boolean(left.compile(ctx)? == right.compile(ctx)?))
                    }
                    BinaryOps::Not(left, right) => {
                        Ok(RawValue::Boolean(!(left.compile(ctx)? == right.compile(ctx)?)))
                    }
                    _ => Ok(RawValue::Undefined)
                }
            }
            _ => Ok(RawValue::Undefined)
        }
    }
}
