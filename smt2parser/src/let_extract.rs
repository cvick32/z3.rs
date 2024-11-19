use std::collections::HashMap;

use crate::{
    concrete::{
        Constant, Keyword, QualIdentifier, SExpr, Sort, Symbol, SyntaxBuilder, Term,
    },
    visitors::TermVisitor,
    CommandStream, Error,
};

#[derive(Clone, Debug, Default)]
pub struct LetExtract {
    pub scope: HashMap<Symbol, Term>,
}
impl LetExtract {
    fn substitute_scoped_symbols(&self, term: Term) -> Term {
        match term {
            Term::Constant(constant) => Term::Constant(constant),
            Term::QualIdentifier(q_id) => {
                let symbol: Symbol = match q_id.clone() {
                    QualIdentifier::Simple { identifier } => match identifier {
                        crate::concrete::Identifier::Simple { symbol } => symbol,
                        crate::concrete::Identifier::Indexed { symbol, indices: _ } => symbol,
                    },
                    QualIdentifier::Sorted {
                        identifier,
                        sort: _,
                    } => match identifier {
                        crate::concrete::Identifier::Simple { symbol } => symbol,
                        crate::concrete::Identifier::Indexed { symbol, indices: _ } => symbol,
                    },
                };
                if self.scope.contains_key(&symbol) {
                    self.scope.get(&symbol).unwrap().clone()
                } else {
                    Term::QualIdentifier(q_id)
                }
            }
            Term::Application {
                qual_identifier,
                arguments,
            } => {
                let new_arguments = arguments
                    .iter()
                    .map(|arg| self.substitute_scoped_symbols(arg.clone()))
                    .collect::<Vec<_>>();
                Term::Application {
                    qual_identifier,
                    arguments: new_arguments,
                }
            }
            Term::Forall { vars, term } => {
                let new_term = self.substitute_scoped_symbols(*term);
                Term::Forall {
                    vars,
                    term: Box::new(new_term),
                }
            }
            Term::Exists { vars, term } => {
                let new_term = self.substitute_scoped_symbols(*term);
                Term::Exists {
                    vars,
                    term: Box::new(new_term),
                }
            }
            Term::Match { term, cases } => {
                let new_term = self.substitute_scoped_symbols(*term);
                let new_cases = cases
                    .iter()
                    .map(|(match_symbols, case)| {
                        (
                            match_symbols.clone(),
                            self.substitute_scoped_symbols(case.clone()),
                        )
                    })
                    .collect::<Vec<_>>();
                Term::Match {
                    term: Box::new(new_term),
                    cases: new_cases,
                }
            }
            Term::Attributes { term, attributes } => {
                let new_term = self.substitute_scoped_symbols(*term);
                Term::Attributes {
                    term: Box::new(new_term),
                    attributes,
                }
            }
            Term::Let { var_bindings, term } => panic!("SHOULD NEVER CALL THIS WITH LET!"),
        }
    }
}

impl TermVisitor<Constant, QualIdentifier, Keyword, SExpr, Symbol, Sort> for LetExtract {
    type T = Term;
    type E = Error;

    fn visit_constant(&mut self, constant: Constant) -> Result<Self::T, Self::E> {
        Ok(Term::Constant(constant))
    }

    fn visit_qual_identifier(
        &mut self,
        qual_identifier: QualIdentifier,
    ) -> Result<Self::T, Self::E> {
        Ok(Term::QualIdentifier(qual_identifier))
    }

    fn visit_application(
        &mut self,
        qual_identifier: QualIdentifier,
        arguments: Vec<Self::T>,
    ) -> Result<Self::T, Self::E> {
        if self.scope.is_empty() {
        Ok(Term::Application {
            qual_identifier,
            arguments,
        })
        } else {
            let new_arguments = arguments
                .iter()
                .map(|arg| self.substitute_scoped_symbols(arg.clone()))
                .collect::<Vec<_>>();
            Ok(Term::Application {
                qual_identifier,
                arguments: new_arguments,
            })
        }
    }

    fn visit_let(
        &mut self,
        var_bindings: Vec<(Symbol, Self::T)>,
        term: Self::T,
    ) -> Result<Self::T, Self::E> {
        for (var, term) in &var_bindings {
            let new_term = self.substitute_scoped_symbols(term.clone());
            self.scope.insert(var.clone(), new_term);
        }
        Ok(self.substitute_scoped_symbols(term))
    }

    fn visit_forall(
        &mut self,
        vars: Vec<(Symbol, Sort)>,
        term: Self::T,
    ) -> Result<Self::T, Self::E> {
        if self.scope.is_empty() {
        Ok(Term::Forall {
            vars,
            term: Box::new(term),
        })
        } else {
            let new_term = self.substitute_scoped_symbols(term);
            Ok(Term::Forall {
                vars,
                term: Box::new(new_term),
            })
        }
    }

    fn visit_exists(
        &mut self,
        vars: Vec<(Symbol, Sort)>,
        term: Self::T,
    ) -> Result<Self::T, Self::E> {
        if self.scope.is_empty() {
        Ok(Term::Exists {
            vars,
            term: Box::new(term),
        })
        } else {
            let new_term = self.substitute_scoped_symbols(term);
            Ok(Term::Exists {
                vars,
                term: Box::new(new_term),
            })
        }
    }

    fn visit_match(
        &mut self,
        term: Self::T,
        cases: Vec<(Vec<Symbol>, Self::T)>,
    ) -> Result<Self::T, Self::E> {
        if self.scope.is_empty() {
        Ok(Term::Match {
            term: Box::new(term),
            cases,
        })
        } else {
            let new_term = self.substitute_scoped_symbols(term);
            Ok(Term::Match {
                term: Box::new(new_term),
                cases,
            })
        }
    }

    fn visit_attributes(
        &mut self,
        term: Self::T,
        attributes: Vec<(
            Keyword,
            crate::concrete::AttributeValue<Constant, Symbol, SExpr>,
        )>,
    ) -> Result<Self::T, Self::E> {
        if self.scope.is_empty() {
        Ok(Term::Attributes {
            term: Box::new(term),
            attributes,
        })
    }
}
