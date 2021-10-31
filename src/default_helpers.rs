
use std::collections::HashMap;
use neon::prelude::{JsObject, Context, Object, JsFunction};
use crate::{compiler::{FnBlockHelper}, convert::{RawValue, IntoRawValue}, error::FinchError, parser::ExpressionKind};

pub fn init() -> HashMap<String, FnBlockHelper> {
    let mut res = HashMap::new();

    res.insert(String::from("each"), FnBlockHelper::Native(|block, ctx| {
        let block_text = block.block.as_ref().unwrap();
         if let RawValue::Vec(var) = block.params[0].compile(ctx)? {
            if let ExpressionKind::Var(name) = &block.params[1] {
                let mut res = String::new();
                ctx.cache.extend();
                for item in var.iter() {
                    ctx.cache.set(name.clone(), item.clone(ctx.cx));
                    res += &block_text.compile(ctx)?;
                }
                ctx.cache.destroy();
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

    res.insert(String::from("js"), FnBlockHelper::Native(|block, ctx| {
        let body = &ctx.original[block.block.as_ref().unwrap().pos.clone()];
        let val = ctx.cx.string(body);
        let param_name = ctx.cx.string("data");
        let func = ctx.cx.global().get(ctx.cx, "Function").map_err(|_| FinchError::None)?.downcast::<JsFunction, _>(ctx.cx).map_err(|_| FinchError::None)?;
        let res = func.construct(ctx.cx, vec![param_name, val]).map_err(|_| FinchError::None)?.downcast::<JsFunction, _>(ctx.cx).map_err(|_| FinchError::None)?;
        let undefined = ctx.cx.undefined();
        let result = res.call(ctx.cx, undefined, vec![ctx.data]).map_err(|er| FinchError::External(er.to_string()))?;
        Ok(result.raw(ctx.cx).to_string())
    }));
    
    res.insert(String::from("if"), FnBlockHelper::Native(|block, ctx| {
        let exp = block.params[0].compile(ctx)?;
        let insides = block.block.as_ref().ok_or(FinchError::ExpectedBody(String::from("if")))?;
        if !exp.is_falsey() {
            return Ok(insides.compile(ctx)?);
        } else if let Some(followup) = &block.chain {
            match followup.name.as_str() {
                "if" => followup.compile(ctx),
                "else" => if let Some(else_bl) = &followup.block {
                    else_bl.compile(ctx)
                } else {
                    Err(FinchError::ExpectedBody(String::from("else")))
                }
                _ => {
                    Err(FinchError::Custom(format!("Expected if / else follow up blocks, found {}", followup.name)))
                }
            }
        } else {
            Ok(String::new())
        }
    }));

    res
}