
use crate::pre_compiler::*;
use std::iter::{Peekable, Enumerate};

pub struct Compiler {
    depth: Vec<PreCompiledTemplate>
}

type Data<'a> = Peekable<Enumerate<std::str::Chars<'a>>>;

impl Compiler {

    fn parse_block(&mut self, data: &mut Data, current_temp: PreCompiledTemplate) -> Option<PreCompiledTemplate> {
        let entry = data.peek()?;
        match entry.1 {
            '#' => {
                let name = Compiler::walk_until(data, ' ')?;
                let params: Vec<Action> = vec![];

                None
            }
            _ => None
        }
    }

    fn parse_action(&mut self, data: &mut Data) -> Option<ActionKind> {
        let entry = data.peek()?;
        match entry.1 {
            '"' => ActionKind::Str(Self::walk_until(data, '"')?),
            _ => {
                let 
            }
        }
    }

    fn walk_until(data: &mut Peekable<Enumerate<std::str::Chars>>, thing: char) -> Option<String> {
        let mut res = String::new();
        while data.peek()?.1 == thing {
            res.push(data.next()?.1);
        };
        Some(res)
    }

}