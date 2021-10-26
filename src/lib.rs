use neon::prelude::*;
mod parser;
mod error;
use parser::{Parser, ExpressionKind, TemplateKind, BinaryOps, UnaryOps};


fn exp_to_str(exp: &ExpressionKind, pos: &std::ops::Range<usize>) -> String {
    match exp {
        ExpressionKind::String(st) => format!("Str: {} ({:?})", st, pos),
        ExpressionKind::Var(v) => format!("Var: {} ({:?})", v, pos),
        ExpressionKind::Number(v) => format!("Num: {} ({:?})", v, pos),
        ExpressionKind::Bool(v) => format!("Bool: {} ({:?})", v, pos),
        ExpressionKind::Undefined => format!("Undefined ({:?})", pos),
        ExpressionKind::Null => format!("Null ({:?})", pos),
        ExpressionKind::VarDot(v) => format!("VarDot: {} ({:?})", v.join("."), pos),
        ExpressionKind::Binary(v) => {
            match &**v {
                BinaryOps::Compare(left, right) => format!("Binary: {} == {}", exp_to_str(&left, pos), exp_to_str(&right, pos)),
                _ => format!("UNKNOWNLOL")
            }
        }
        ExpressionKind::Unary(v) => {
            match &**v {
                UnaryOps::Not(exp) => format!("Unary NOT: !{}", exp_to_str(exp, pos))
            }
        }
        _ => String::from("Some expression IDK")
    }
}

fn hello(mut cx: FunctionContext) -> JsResult<JsString> {
    let res = Parser::parse(&cx.argument::<JsString>(0)?.value(&mut cx)).unwrap();
    println!("{:?} {}", res.1.templates.iter().map(|i| {
        match &i.kind {
            TemplateKind::Expression(exp) => exp_to_str(exp, &i.pos),
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
