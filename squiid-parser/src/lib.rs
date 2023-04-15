use logos::Logos;

#[derive(Logos, Debug, PartialEq)]
#[logos(skip r"[ \t\n\f]+")]
#[logos(subpattern identifier=r"[_a-zA-Z][_0-9a-zA-Z]*")]
#[logos(subpattern float=r"[0-9]+\.[0-9]+")]
pub enum Token {

    // identifier followed by optional spaces followed by an opening parenthesis
    // will match everything until the closing parenthesis
    #[regex(r"(?&identifier)\s*\([^()]*\)")]
    Function,

    // identifier
    #[regex(r"(?&identifier)")]
    VariableAssign,
    // $ followed by identifier 
    #[regex(r"\$(?&identifier)")]
    VariableRecal,

    // optional negative sign
    // optional float
    // an e followed by an option + or -
    // 1 or more digits (the number following the e)
    // an optional decimal point followed by 1 or more digits (3.1) or (.1)
    #[regex(r"((?&float)?([eE][-+]?\d+(\.\d+)?))", priority=3)]
    ScientificNotation,
    #[regex("(?&float)+", priority=2)]
    Float,
    #[regex(r"[0-9]+", priority=1)]
    Int,

    #[token("@")]
    PrevAns,

    #[token("(")]
    LParen,
    #[token(")")]
    RParen,
    #[token("=")]
    Equal,
    #[token("^")]
    Power,
    #[token("*")]
    Multiply,
    #[token("/")]
    Divide,
    #[token("+")]
    Add,
    // this can be the unary operator (-3) or the binary operator (3-4)
    #[token("-")]
    Subtract,
}

// main shunting-yard parsing function
pub fn parse(input: &str) -> () {
    let lex = Token::lexer(input);
    
    for (token, range) in lex.spanned() {
        match token {
            Ok(item) => {
                println!("{:?}", item);
            },
            Err(_) => println!("Unexpected token: {:?}", &input[range.start..range.end]),
        }
    }
}