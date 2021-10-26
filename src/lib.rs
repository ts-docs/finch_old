use neon::prelude::*;
mod parser;
mod error;
use parser::{Parser, ExpressionKind, TemplateKind};

fn hello(mut cx: FunctionContext) -> JsResult<JsString> {
    let res = Parser::parse(&cx.argument::<JsString>(0)?.value(&mut cx)).unwrap();
    println!("{:?} {}", res.1.templates.iter().map(|i| {
        match &i.kind {
            TemplateKind::Expression(exp) => {
                match &exp {
                    ExpressionKind::String(st) => format!("Str: {} ({:?})", st, i.pos),
                    ExpressionKind::Var(v) => format!("Var: {} ({:?})", v, i.pos),
                    ExpressionKind::Number(v) => format!("Num: {} ({:?})", v, i.pos),
                    ExpressionKind::Bool(v) => format!("Bool: {} ({:?})", v, i.pos),
                    ExpressionKind::Undefined => format!("Undefined ({:?})", i.pos),
                    ExpressionKind::Null => format!("Null ({:?})", i.pos),
                    ExpressionKind::VarDot(v) => format!("VarDot: {} ({:?})", v.join("."), i.pos),
                    _ => String::from("Some expression IDK")
                }
            }
            _ => String::from("block")
        }
    }).collect::<Vec<String>>().join(", "), res.1.templates.len());
    Ok(cx.string("hello node"))
}

#[neon::main]
fn main(mut cx: ModuleContext) -> NeonResult<()> {
    cx.export_function("hello", hello)?;
    Ok(())
}
