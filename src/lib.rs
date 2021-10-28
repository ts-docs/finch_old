use neon::prelude::*;
mod parser;
mod error;
mod convert;
mod compiler;
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
        COMPILER.lock().unwrap().add_template(&name, &value);
        Ok(cx.undefined())
    })?;
    cx.export_function("compile", |mut cx: FunctionContext| -> JsResult<JsString> {
        match COMPILER.lock().unwrap().compile(&mut cx) {
            Ok(res) => Ok(cx.string(res)),
            Err(err) => Ok(cx.string(format!("{}", err)))
        }
    })?;
    Ok(())
}
