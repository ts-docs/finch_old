

pub enum ActionKind {
    Dot(Vec<String>),
    Str(String),
    Num(f32),
    Bool(bool),
    HelperWithBody(String, Vec<ActionKind>, Box<PreCompiledTemplate>),
    Helper(String, Vec<ActionKind>)
}

pub struct Action {
    kind: ActionKind,
    pos: usize
}

pub struct PreCompiledTemplate {
    actions: Vec<Action>,
    data: Vec<char>
}