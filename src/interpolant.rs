use smt2parser::{
    concrete::{SyntaxBuilder, Term},
    vmt::numbered_to_symbolic::NumberedToSymbolic,
};

pub struct Interpolant {
    _original_term: Term,
    _new_term: Term,
}

impl Interpolant {
    pub fn from(term: &Term, interpolant_number: usize) -> Self {
        let mut builder = NumberedToSymbolic {
            visitor: SyntaxBuilder,
            step: interpolant_number,
        };
        let new_term = term.clone().accept(&mut builder).unwrap();
        println!("{}", new_term);
        Interpolant {
            _original_term: term.clone(),
            _new_term: new_term,
        }
    }
}
