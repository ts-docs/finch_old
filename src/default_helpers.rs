
use std::collections::HashMap;
use crate::{compiler::{FnBlockHelper, Compilable}, convert::RawValue, error::FinchError};

pub fn init() -> HashMap<String, FnBlockHelper> {
    let mut res = HashMap::new();

    res.insert(String::from("each"), FnBlockHelper::Native(|block, ctx| {
        let block_text = block.block.as_ref().unwrap();
         if let RawValue::Vec(var) = block.params[0].compile(ctx)? {
            if let RawValue::String(name) = block.params[1].compile(ctx)? {
                let mut res = String::new();
                for item in var.iter() {
                    ctx.locals.insert(name.clone(), item.clone());
                    res += &block_text.compile(ctx)?;
                }
                ctx.locals.remove(&name);
                Ok(res)
            } else { Err(FinchError::InvalidArg(1)) }
         } else { Err(FinchError::InvalidArg(0)) }
    }));

    res
}