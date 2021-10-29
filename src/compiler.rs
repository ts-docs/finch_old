
use crate::parser::*;
use crate::error::FinchError;
use crate::convert::*;
use neon::types::{JsObject, JsString, JsValue, JsFunction};
use neon::handle::{Handle, Root};
use neon::object::Object;
use neon::context::{FunctionContext, Context};
use std::collections::HashMap;
use crate::default_helpers;

pub enum FnBlockHelper {
    Native(fn(block: &FnBlock, cx: &mut CompilerContext) -> FinchResult<String>),
    Js(Root<JsFunction>)
}

pub struct Compiler {
    pub templates: HashMap<String, (String, SubText)>,
    pub helpers: HashMap<String, FnBlockHelper>
}

pub struct CompilerContext<'a, 'b> {
    pub compiler: &'a Compiler,
    pub cx: &'a mut FunctionContext<'b>,
    pub locals: HashMap<String, RawValue>,
    pub data: &'a Handle<'a, JsObject>,
    pub original: &'a str
}

impl Compiler {

    pub fn new() -> Self {
        Self { 
            templates: HashMap::new(),
            helpers: default_helpers::init()
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
            compiler: self,
            cx,
            locals: HashMap::new(),
            data: &data,
            original: og
        })
    }

}

pub trait Compilable<T> {
    fn compile<'a, 'b>(&self, ctx: &mut CompilerContext<'a, 'b>) -> FinchResult<T>;
}

impl<'a> Compilable<String> for SubText {

    fn compile(&self, ctx: &mut CompilerContext) -> FinchResult<String> {
        let mut res = String::new();
        let mut last_temp_end = self.pos.start;
        for temp in &self.templates {
            let temp_str = match &temp.kind {
                TemplateKind::Expression(exp) => exp.compile(ctx)?.into_string(),
                TemplateKind::Block(bl) => {
                    if let Some(func) = ctx.compiler.helpers.get(&bl.name) {
                        match func {
                            FnBlockHelper::Native(function) => function(bl, ctx)?,
                            _ => String::new()
                        }
                    } else { String::new() }
                }
            };
            if last_temp_end < temp.pos.start {
                res += &ctx.original[last_temp_end..temp.pos.start];
            }
            last_temp_end = temp.pos.end;
            res += &temp_str;
        }
        if self.pos.end > last_temp_end {
            res += &ctx.original[last_temp_end..self.pos.end];
        }
        Ok(res)
    }

}

impl ExpressionKind {

    fn compile_to_js<'a, 'b>(&self, ctx: &mut CompilerContext<'a, 'b>) -> FinchResult<Handle<'b, JsValue>> {
        match self {
            ExpressionKind::String(val) => Ok(ctx.cx.string(val.as_str()).upcast::<JsValue>()),
            ExpressionKind::Number(num) => Ok(ctx.cx.number(*num).upcast::<JsValue>()),
            ExpressionKind::Bool(bol) => Ok(ctx.cx.boolean(*bol).upcast::<JsValue>()),
            ExpressionKind::Undefined => Ok(ctx.cx.undefined().upcast::<JsValue>()),
            ExpressionKind::Null => Ok(ctx.cx.null().upcast::<JsValue>()),
            ExpressionKind::Var(val) => Ok(ctx.data.get(ctx.cx, val.as_str()).map_err(|_| FinchError::PropNotExist(val.to_string()))?),
            ExpressionKind::VarDot(path) => {
                let first = &path[0];
                let mut dat = ctx.data.get(ctx.cx, first.as_str()).map_err(|_| FinchError::PropNotExist(first.to_string()))?;
                for item in path.iter().skip(1) {
                    dat = dat.downcast::<JsObject, _>(ctx.cx).map_err(|_| FinchError::ExpectedObject)?.get(ctx.cx, item.as_str()).map_err(|_| FinchError::ExpectedObject)?;
                };
                Ok(dat)
            },
            ExpressionKind::Unary(_) | ExpressionKind::Binary(_) => Ok(self.compile(ctx)?.js(ctx.cx)),
            ExpressionKind::Call{var: _, params: _} => Ok(self.compile(ctx)?.js(ctx.cx))
        }
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
                for item in path.iter().skip(1) {
                    dat = dat.downcast::<JsObject, _>(ctx.cx).map_err(|_| FinchError::ExpectedObject)?.get(ctx.cx, item.as_str()).map_err(|_| FinchError::ExpectedObject)?;
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
            },
            ExpressionKind::Call{var, params} => {
                let mut mapped_params: Vec<Handle<JsValue>> = vec![];
                for param in params {
                    mapped_params.push(param.compile_to_js(ctx)?)
                }
                match &**var {
                    ExpressionKind::Var(name) => {
                        let func = ctx.data.get(ctx.cx, name.as_str()).map_err(|_| FinchError::PropNotExist(name.to_string()))?;
                        if let Ok(val) = func.downcast::<JsFunction, _>(ctx.cx) {
                            let undefiend = ctx.cx.undefined();
                            let return_val = val.call(ctx.cx, undefiend, mapped_params).map_err(|_| FinchError::ErrInFunction(name.to_string()))?;
                            Ok(return_val.raw(ctx.cx))
                        } else {
                            Err(FinchError::NotCallable(name.to_string()))
                        }
                    }
                    _ => Err(FinchError::None)
                }
            }
        }
    }
}
