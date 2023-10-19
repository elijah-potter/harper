use lt_core::{lex_to_end, Lint, TokenKind};

fn main() {
    let txt = include_str!("../../alice.txt");
    let chars: Vec<_> = txt.chars().collect();

    let tokens = lex_to_end(&chars);

    for token in tokens.iter() {
        match token.kind {
            TokenKind::Number(n) => println!("{}", Lint::new(token.span).display(&chars)),
            _ => (),
        }
    }

    //     for token in tokens {
    //      match token.kind {
    //          TokenKind::Word => println!(
    //              "\"{}\"",
    //              token.span.get_content(&chars).iter().collect::<String>()
    //          ),
    //          TokenKind::Number(n) => println!("${}$", n),
    //          TokenKind::Punctuation(_) => println!(
    //              "_{}_",
    //              token.span.get_content(&chars).iter().collect::<String>()
    //          ),
    //      }
    //  }
}
