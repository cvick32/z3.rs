use std::collections::HashMap;

use crate::concrete::{SyntaxBuilder, Term};

use super::utils::get_function_name;

/// CV: Currently I'm thinking that we don't care at all about boolean functions for
/// this use case. Adding a bunch of True/False values into the egraph will probably
/// just bog stuff down. We don't care about Reads and Writes because we're already
/// adding the full interpretation from the model.
static DONT_CARE_FUNCTIONS: [&str; 11] = [
    "Read-Int-Int",
    "Write-Int-Int",
    "and",
    "or",
    "=>",
    "=",
    "not",
    "<",
    ">",
    ">=",
    "<=",
];

#[derive(Clone, Default)]
pub struct TermExtractor {
    pub visitor: SyntaxBuilder,
    pub current_to_next_variables: HashMap<String, String>,
    pub terms: Vec<Term>,
}

impl crate::rewriter::Rewriter for TermExtractor {
    type V = SyntaxBuilder;
    type Error = crate::concrete::Error;

    fn visitor(&mut self) -> &mut Self::V {
        &mut self.visitor
    }

    fn visit_application(
        &mut self,
        qual_identifier: <Self::V as crate::visitors::Smt2Visitor>::QualIdentifier,
        arguments: Vec<<Self::V as crate::visitors::Smt2Visitor>::Term>,
    ) -> Result<<Self::V as crate::visitors::Smt2Visitor>::Term, Self::Error> {
        let function_name = get_function_name(qual_identifier.clone());
        let term = Term::Application {
            qual_identifier: qual_identifier.clone(),
            arguments: arguments.clone(),
        };
        if !DONT_CARE_FUNCTIONS.contains(&function_name.as_str()) {
            self.terms.push(term.clone());
        }
        Ok(term)
    }
}
