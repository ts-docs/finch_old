
use crate::parser::*;
use crate::error::FinchError;
use crate::convert::*;
use neon::types::{JsObject, JsString, JsNumber};
use neon::handle::Handle;
use neon::object::Object;
use neon::context::FunctionContext;
use std::collections::HashMap;

pub struct Compiler<'a> {
    config: Handle<'a, JsObject>,
    locals: HashMap<String, RawValue>,
    templates: HashMap<String, (String, SubText)>,
}

impl<'a> Compiler<'a> {

    pub fn new(shared_config: Handle<'a, JsObject>) -> Self {
        Self { 
            config: shared_config, 
            templates: HashMap::new(),
            locals: HashMap::new()
         }
    }

    pub fn add_template(&mut self, name: &'a str, text: &'a str) -> FinchResult<()> {
        let parsed = Parser::parse(text)?;
        self.templates.insert(name.to_string(), (text.to_string(), parsed));
        Ok(())
    }

    pub fn compile(&self, cx: &mut FunctionContext) -> FinchResult<String> {
        let name = cx.argument::<JsString>(0).map_err(|_| FinchError::InvalidArg(0))?.value(cx);
        let data= cx.argument::<JsObject>(1).map_err(|_| FinchError::InvalidArg(1))?;
        let temp = self.templates.get(&name).ok_or(FinchError::TemplateNotExist(name))?;
        Err(FinchError::None)
    }

}

pub trait Compilable<T> {
    fn compile(&self, cx: &mut FunctionContext, compiler: &mut Compiler, original: &str, data: &Handle<JsObject>) -> FinchResult<T>;
}

pub trait CompilableShortCircuit<T> {
    fn compile_short_circuit(&self, cx: &mut FunctionContext, compiler: &mut Compiler, original: &str, data: &Handle<JsObject>) -> FinchResult<T>;
}

impl Compilable<RawValue> for ExpressionKind {

    fn compile(&self, cx: &mut FunctionContext, compiler: &mut Compiler, _original: &str, data: &Handle<JsObject>) -> FinchResult<RawValue> {
        match self {
            ExpressionKind::String(val) => Ok(RawValue::String(val.to_string())),
            ExpressionKind::Bool(val) => Ok(RawValue::Boolean(*val)),
            ExpressionKind::Number(val) => Ok(RawValue::Number(*val)),
            ExpressionKind::Null => Ok(RawValue::Null),
            ExpressionKind::Undefined => Ok(RawValue::Undefined),
            ExpressionKind::Var(val) => {
                if let Some(thing) = compiler.locals.get(val) {
                    return Ok(thing.clone());
                }
                let dat = data.get(cx, val.as_str()).map_err(|_| FinchError::PropNotExist(val.to_string()))?;
                if let Ok(str_handle) = dat.downcast::<JsString, _>(cx) {
                    let str_val = RawValue::String(str_handle.value(cx));
                    Ok(str_val)
                } else if let Ok(num_handle) = dat.downcast::<JsNumber, _>(cx) {
                    let str_val = RawValue::Number(num_handle.value(cx));
                    Ok(str_val)
                } else {
                    Err(FinchError::None)
                }
            }
        }
    }
}
