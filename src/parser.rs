use crate::Args;
use crate::Selection;
use crate::tokenizer::TokenList;

pub fn parse(sel: &Selection, tokens: &TokenList) -> (Vec<String>, Args) {
    let mut sel = sel;
    let mut result = Vec::new();
    for token in tokens {
        match sel {
            // TODO token.quoted abfragen
            Selection::Fixed(h) => {
                if let Some(s) = h.get(&token.text) {
                    result.push(token.text.clone());
                    sel = s;
                } else {
                    eprintln!("*** not found: {}", token.text);
                    break;
                }
            }
            _ => break,
        }
    }
    if !matches!(sel, Selection::End) {
        eprintln!("*** missing...");
    }
    (result, Args {})
}
