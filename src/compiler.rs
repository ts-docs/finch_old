
use crate::parser::*;
use crate::error::FinchError;
use crate::convert::*;
use crate::memory::*;
use neon::types::{JsObject, JsValue, JsFunction, JsArray};
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
    pub cache: Memory,
    pub data: Handle<'a, JsObject>,
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

    pub fn add_helper(&mut self, name: String, func: Root<JsFunction>) {
        self.helpers.insert(name, FnBlockHelper::Js(func));
    }

    pub fn compile(&self, cx: &mut FunctionContext, name: &str, data: Handle<JsObject>) -> FinchResult<String> {
        let (og, temp) = self.templates.get(name).ok_or(FinchError::TemplateNotExist(name.to_string()))?;
        temp.compile(&mut CompilerContext {
            compiler: self,
            cx,
            cache: Memory::new(),
            data: data,
            original: og
        })
    }

}

impl<'a> SubText {

    pub fn compile(&self, ctx: &mut CompilerContext) -> FinchResult<String> {
        let mut res = String::new();
        let mut last_temp_end = self.pos.start;
        for temp in &self.templates {
            let temp_str = match &temp.kind {
                TemplateKind::Expression(exp) => exp.compile(ctx)?.into_string(),
                TemplateKind::Block(bl) => bl.compile(ctx)?
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

impl FnBlock {
    pub fn compile(&self, ctx: &mut CompilerContext) -> FinchResult<String> {
        if let Some(func) = ctx.compiler.helpers.get(&self.name) {
            match func {
                FnBlockHelper::Native(function) => function(self, ctx),
                FnBlockHelper::Js(func) => {
                    let func = func.to_inner(ctx.cx);
                    let args_arr = JsArray::new(ctx.cx, self.params.len() as u32);
                    for (ind, param) in self.params.iter().enumerate() {
                        let param_js = param.compile_to_js(ctx)?;
                        args_arr.set(ctx.cx, ind as u32, param_js).map_err(|er| FinchError::External(er.to_string()))?;
                    }
                    let body = if let Some(b) = &self.block {
                        let compiled_block = b.compile(ctx)?;
                        ctx.cx.string(compiled_block).upcast::<JsValue>()
                    } else { ctx.cx.undefined().upcast::<JsValue>() };
                    let undefined = ctx.cx.undefined();
                    let res = func.call(ctx.cx, undefined, vec![args_arr.upcast::<JsValue>(), body]).map_err(|er| FinchError::External(er.to_string()))?;
                    Ok(res.raw(ctx.cx).to_string())
                }
            }
        } else { Err(FinchError::HelperNotFound(self.name.to_string())) }
    }
}

impl ExpressionKind {

    pub fn compile_to_js<'a, 'b>(&self, ctx: &mut CompilerContext<'a, 'b>) -> FinchResult<Handle<'b, JsValue>> {
        match self {
            ExpressionKind::String(val) => Ok(ctx.cx.string(val.as_str()).upcast::<JsValue>()),
            ExpressionKind::Number(num) => Ok(ctx.cx.number(*num).upcast::<JsValue>()),
            ExpressionKind::Bool(bol) => Ok(ctx.cx.boolean(*bol).upcast::<JsValue>()),
            ExpressionKind::Undefined => Ok(ctx.cx.undefined().upcast::<JsValue>()),
            ExpressionKind::Null => Ok(ctx.cx.null().upcast::<JsValue>()),
            ExpressionKind::Var(val) => {
                if let Some(thing) = ctx.cache.get(val) {
                    return Ok(thing.js(ctx.cx));
                } 
                Ok(ctx.data.get(ctx.cx, val.as_str()).map_err(|_| FinchError::PropNotExist(val.to_string()))?)
            },
            ExpressionKind::VarDot(path) => {
                let first = &path[0];
                let mut dat = if let Some(thing) = ctx.cache.get(first) {
                    thing.js(ctx.cx)
                } else if let Some(thing) = ctx.cache.get(&path.join(".")) {
                    thing.js(ctx.cx)
                } else {
                    ctx.data.get(ctx.cx, first.as_str()).map_err(|_| FinchError::PropNotExist(first.to_string()))?
                };
                for item in path.iter().skip(1) {
                    dat = dat.downcast::<JsObject, _>(ctx.cx).map_err(|_| FinchError::ExpectedObject)?.get(ctx.cx, item.as_str()).map_err(|_| FinchError::ExpectedObject)?;
                };
                Ok(dat)
            },
            ExpressionKind::Unary(_) | ExpressionKind::Binary(_) => Ok(self.compile(ctx)?.js(ctx.cx)),
            ExpressionKind::Call{var: _, params: _} => Ok(self.compile(ctx)?.js(ctx.cx))
        }
    }

    pub fn compile(&self, ctx: &mut CompilerContext) -> FinchResult<RawValue> {
        match self {
            ExpressionKind::String(val) => Ok(RawValue::String(val.to_string())),
            ExpressionKind::Bool(val) => Ok(RawValue::Boolean(*val)),
            ExpressionKind::Number(val) => Ok(RawValue::Number(*val)),
            ExpressionKind::Null => Ok(RawValue::Null),
            ExpressionKind::Undefined => Ok(RawValue::Undefined),
            ExpressionKind::Var(val) => {
                if let Some(thing) = ctx.cache.get(val) {
                    return Ok(thing.clone(ctx.cx));
                }
                let dat = ctx.data.get(ctx.cx, val.as_str()).map_err(|_| FinchError::PropNotExist(val.to_string()))?.raw(ctx.cx);
                ctx.cache.set(val.to_string(), dat.clone(ctx.cx));
                Ok(dat)
            },
            ExpressionKind::VarDot(path) => {
                let first = &path[0];
                let joined = path.join(".");
                let mut dat = if let Some(thing) = ctx.cache.get(first) {
                    thing.js(ctx.cx)
                } else if let Some(thing) = ctx.cache.get(&joined) {
                    thing.js(ctx.cx)
                }
                else {
                    ctx.data.get(ctx.cx, first.as_str()).map_err(|_| FinchError::PropNotExist(first.to_string()))?
                };
                for item in path.iter().skip(1) {
                    dat = dat.downcast::<JsObject, _>(ctx.cx).map_err(|_| FinchError::ExpectedObject)?.get(ctx.cx, item.as_str()).map_err(|_| FinchError::ExpectedObject)?;
                };
                let raw_thing = dat.raw(ctx.cx);
                ctx.cache.set(joined, raw_thing.clone(ctx.cx));
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
