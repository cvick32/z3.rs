/// Implement Smt2Parser that takes in standard Array based VMT and returns smtlib that has array theory abstracted away. 
/// Add:
///   Arr sort definition
///   Write definition
///   Read defintiion
///   ConstArr definition
/// Replace: 
///   store  -> Write
///   select -> Read
///   K      -> ConstArr
/// 
/// 

use smt2parser::{concrete::{Error, SyntaxBuilder, Symbol}, rewriter::Rewriter};


#[derive(Default)]
struct Builder(SyntaxBuilder);
impl Rewriter for Builder {
    type V = SyntaxBuilder;
    type Error = Error;

    fn visitor(&mut self) -> &mut Self::V {
        &mut self.0
    }

    fn process_symbol(&mut self, s: Symbol) -> Result<Symbol, Self::Error> {
        Ok(Symbol(s.0 + "__"))
    }
}