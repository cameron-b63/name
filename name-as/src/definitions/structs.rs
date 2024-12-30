use super::expandables::ExpansionFn;

#[derive(Debug, Clone, PartialEq)]
pub enum LineComponent {
    Mnemonic(String),
    Register(String),
    Immediate(i32),
    Identifier(String),
    Label(String),
    Directive(String),
    DoubleQuote(String),
    Colon,
}

#[derive(Debug)]
pub(crate) struct PseudoInstruction {
    pub(crate) mnemonic: &'static str,
    pub(crate) expand: ExpansionFn,
}
