use neon::prelude::*;
mod parser;
mod error;
use parser::{Parser, ExpressionKind, TemplateKind, BinaryOps, UnaryOps};


fn exp_to_str(exp: &ExpressionKind) -> String {
    match exp {
        ExpressionKind::String(st) => format!("\"{}\"", st),
        ExpressionKind::Var(v) => format!("{}", v),
        ExpressionKind::Number(v) => format!("{}", v),
        ExpressionKind::Bool(v) => format!("{}", v),
        ExpressionKind::Undefined => format!("undefined"),
        ExpressionKind::Null => format!("null"),
        ExpressionKind::VarDot(v) => format!("{}", v.join(".")),
        ExpressionKind::Binary(v) => {
            match &**v {
                BinaryOps::Compare(left, right) => format!("({} == {})", exp_to_str(&left), exp_to_str(&right)),
                BinaryOps::Not(left, right) => format!("({} != {})", exp_to_str(&left), exp_to_str(&right)),
                BinaryOps::And(left, right) => format!("({} && {})", exp_to_str(&left), exp_to_str(&right)),
                BinaryOps::Or(left, right) => format!("({} || {})", exp_to_str(&left), exp_to_str(&right)),
                _ => format!("UNKNOWNLOL")
            }
        }
        ExpressionKind::Unary(v) => {
            match &**v {
                UnaryOps::Not(exp) => format!("!{}", exp_to_str(exp))
            }
        },
        ExpressionKind::Call{var, params} => format!("{}({})", exp_to_str(var), params.iter().map(|v| exp_to_str(v)).collect::<Vec<String>>().join(", "))
    }
}

fn hello(mut cx: FunctionContext) -> JsResult<JsString> {
    let res = Parser::parse(&cx.argument::<JsString>(0)?.value(&mut cx)).unwrap();
    println!("{:?} {}", res.1.templates.iter().map(|i| {
        match &i.kind {
            TemplateKind::Expression(exp) => exp_to_str(exp),
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
