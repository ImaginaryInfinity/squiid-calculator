use logos::Logos;

#[derive(Logos, Debug)]
#[logos(skip r"[ \t\n\f]+")]
#[logos(subpattern identifier=r"[_a-zA-Z][_0-9a-zA-Z]*")]
#[logos(subpattern float=r"[0-9]+\.[0-9]+")]
pub enum Token<'a> {
    /// Identifier followed by optional spaces followed by an opening parenthesis
    #[regex(r"(?&identifier)\s*\(")]
    Function(&'a str),
    #[token(",")]
    Comma(&'a str),

    /// Identifier
    #[regex(r"(?&identifier)")]
    VariableAssign(&'a str),
    /// $ followed by identifier
    #[regex(r"\$(?&identifier)")]
    VariableRecal(&'a str),
    /// # followed by identifier
    #[regex(r"#(?&identifier)")]
    Constant(&'a str),

    /// optional int/float
    ///
    /// an e followed by an option + or -
    ///
    /// 1 or more digits (the number following the e)
    ///
    /// an optional decimal point followed by 1 or more digits (3.1) or (.1)
    #[regex(r"[0-9]*\.?[0-9]+([eE][-+]?\d+(\.\d+)?)", priority = 3)]
    ScientificNotation(&'a str),
    #[regex("(?&float)+", priority = 2)]
    Float(&'a str),
    #[regex(r"[0-9]+", priority = 1)]
    Int(&'a str),

    /// An @ signifies the previous answer
    #[token("@")]
    PrevAns(&'a str),

    #[token("(")]
    LParen(&'a str),
    #[token(")")]
    RParen(&'a str),
    #[token("=")]
    Equal(&'a str),
    #[token("^")]
    Power(&'a str),
    #[token("*")]
    Multiply(&'a str),
    #[token("/")]
    Divide(&'a str),
    #[token("%")]
    Modulo(&'a str),
    #[token("+")]
    Add(&'a str),
    // this can be the unary operator (-3) or the binary operator (3-4)
    #[token("-")]
    Subtract(&'a str),

    // Logic Operators
    #[token(">")]
    GreaterThan(&'a str),
    #[token("<")]
    LessThan(&'a str),
    #[token(">=")]
    GreaterThanEqualTo(&'a str),
    #[token("<=")]
    LessThanEqualTo(&'a str),
    #[token("==")]
    EqualTo(&'a str),


    /// This cannot be a token, it is used for differentiation between minus and negative later on in parsing
    Negative(&'a str),
}

/// PartialEq implementation that ignores the content of the enum
impl<'a> PartialEq for Token<'a> {
    fn eq(&self, other: &Self) -> bool {
        std::mem::discriminant(self) == std::mem::discriminant(other)
    }

    fn ne(&self, other: &Self) -> bool {
        std::mem::discriminant(self) != std::mem::discriminant(other)
    }
}
