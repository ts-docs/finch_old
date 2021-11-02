use neon::prelude::*;
mod parser;
mod error;
mod convert;
mod default_helpers;
mod memory;
mod compiler;
mod transpiler;
use compiler::Compiler;
use std::{sync::Mutex};

#[macro_use]
extern crate lazy_static;

lazy_static! {
    static ref COMPILER: Mutex<Compiler> = Mutex::new(Compiler::new());
}

#[neon::main]
fn main(mut cx: ModuleContext) -> NeonResult<()> {

    cx.export_function("addTemplate", |mut cx: FunctionContext| -> JsResult<JsUndefined> {
        let name = cx.argument::<JsString>(0)?.value(&mut cx);
        let value = cx.argument::<JsString>(1)?.value(&mut cx);
        COMPILER.lock().unwrap().add_template(&name, &value).unwrap();
        Ok(cx.undefined())
    })?;

    cx.export_function("compile", |mut cx: FunctionContext| -> JsResult<JsString> {
        let name = cx.argument::<JsString>(0)?.value(&mut cx);
        let data= cx.argument::<JsObject>(1)?;
        match COMPILER.lock().unwrap().compile(&mut cx, &name, data) {
            Ok(res) => Ok(cx.string(res)),
            Err(err) => panic!("{}", err)
        }
    })?;
    
    cx.export_function("addHelper", |mut cx: FunctionContext| -> JsResult<JsUndefined> {
        let name = cx.argument::<JsString>(0)?.value(&mut cx);
        let value = cx.argument::<JsFunction>(1)?.root(&mut cx);
        COMPILER.lock().unwrap().add_helper(name, value);
        Ok(cx.undefined())
    })?;

    cx.export_function("removeHelper", |mut cx: FunctionContext| -> JsResult<JsUndefined> {
        let name = cx.argument::<JsString>(0)?.value(&mut cx);
        COMPILER.lock().unwrap().helpers.remove(&name);
        Ok(cx.undefined())
    })?;

    Ok(())
}
