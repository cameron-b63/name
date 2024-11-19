pub enum Ast {
    Symbol(String),
    Number(u32),
    Register(Register),
    BaseAddress(u32, Box<Ast>),
    Instruction(String, Vec<Ast>),
    Directive(String, Vec<Ast>),
}
