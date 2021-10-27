use neon::prelude::*;
mod parser;
mod error;
use parser::{BinaryOps, ExpressionKind, FnBlock, Parser, TemplateKind, UnaryOps};


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
                BinaryOps::Compare(left, right) => format!("({} == {})", exp_to_str(left), exp_to_str(right)),
                BinaryOps::Not(left, right) => format!("({} != {})", exp_to_str(left), exp_to_str(right)),
                BinaryOps::And(left, right) => format!("({} && {})", exp_to_str(left), exp_to_str(right)),
                BinaryOps::Or(left, right) => format!("({} || {})", exp_to_str(left), exp_to_str(right)),
                BinaryOps::Gt(left, right) => format!("({} > {})", exp_to_str(left), exp_to_str(right)),
                BinaryOps::Gte(left, right) => format!("({} >= {})", exp_to_str(left), exp_to_str(right)),
                BinaryOps::Lt(left, right) => format!("({} < {})", exp_to_str(left), exp_to_str(right)),
                BinaryOps::Lte(left, right) => format!("({} <= {})", exp_to_str(left), exp_to_str(right))
            }
        }
        ExpressionKind::Unary(v) => {
            match &**v {
                UnaryOps::Not(exp) => format!("!{}", exp_to_str(exp)),
                UnaryOps::ShortCircuit(exp) => format!("{}?", exp_to_str(exp))
            }
        },
        ExpressionKind::Call{var, params} => format!("{}({})", exp_to_str(var), params.iter().map(|v| exp_to_str(v)).collect::<Vec<String>>().join(", "))
    }
}

fn block_to_str(bl: &FnBlock) -> String {
    format!("
Block: {},
Params: {},
Inside: {},
Chain: {}
", bl.name, bl.params.iter().map(|v| exp_to_str(v)).collect::<Vec<String>>().join(", "), if let Some(block) = &bl.block { format!("{} ({:?})", block.templates.iter().map(|v| template_to_str(&v.kind)).collect::<Vec<String>>().join("\n"), block.pos) } else { String::from("None") }, if bl.chain.is_some() { block_to_str(bl.chain.as_ref().unwrap()) } else { String::from("None") } )
}

fn template_to_str(temp: &TemplateKind) -> String {
    match &temp {
        TemplateKind::Expression(exp) => exp_to_str(&exp),
        TemplateKind::Block(bl) => block_to_str(&bl)
    }
}

fn hello(mut cx: FunctionContext) -> JsResult<JsString> {
    let res = Parser::parse(&cx.argument::<JsString>(0)?.value(&mut cx)).unwrap();
    println!("{}\n{:?}", res.templates.iter().map(|i| template_to_str(&i.kind)).collect::<Vec<String>>().join("\n"), res.pos);
    Ok(cx.string("hello node"))
}

#[neon::main]
fn main(mut cx: ModuleContext) -> NeonResult<()> {
    cx.export_function("hello", hello)?;
    Ok(())
}
