
use std::collections::HashMap;
use neon::prelude::JsObject;
use crate::{compiler::{FnBlockHelper}, convert::RawValue, error::FinchError};

pub fn init() -> HashMap<String, FnBlockHelper> {
    let mut res = HashMap::new();

    res.insert(String::from("each"), FnBlockHelper::Native(|block, ctx| {
        let block_text = block.block.as_ref().unwrap();
         if let RawValue::Vec(var) = block.params[0].compile(ctx)? {
            if let RawValue::String(name) = block.params[1].compile(ctx)? {
                let mut res = String::new();
                for item in var.iter() {
                    ctx.locals.insert(name.clone(), item.clone(ctx.cx));
                    res += &block_text.compile(ctx)?;
                }
                ctx.locals.remove(&name);
                Ok(res)
            } else { Err(FinchError::InvalidArg(1)) }
         } else { Err(FinchError::InvalidArg(0)) }
    }));

    res.insert(String::from("template"), FnBlockHelper::Native(|block, ctx| {
        if block.block.is_some() { panic!("template block cannot have body") };
        if let RawValue::String(temp_name) = block.params[0].compile(ctx)? {
            let data = block.params[1].compile_to_js(ctx)?.downcast::<JsObject, _>(ctx.cx).map_err(|er| FinchError::External(er.to_string()))?;
            Ok(ctx.compiler.compile(ctx.cx, &temp_name, data)?)
        } else {
            Err(FinchError::InvalidArg(0))
        }
    }));

    res
}