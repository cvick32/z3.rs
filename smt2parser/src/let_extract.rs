use crate::{
    concrete::{Constant, Keyword, QualIdentifier, SExpr, Sort, Symbol, Term},
    visitors::TermVisitor,
    Error,
};

#[derive(Clone, Debug, Default)]
pub struct LetExtract {
    pub terms: Vec<Term>,
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
        Ok(Term::Application {
            qual_identifier,
            arguments,
        })
    }

    fn visit_let(
        &mut self,
        var_bindings: Vec<(Symbol, Self::T)>,
        term: Self::T,
    ) -> Result<Self::T, Self::E> {
        for (_var, term) in &var_bindings {
            self.terms.push(term.clone());
        }
        Ok(Term::Let {
            var_bindings,
            term: Box::new(term),
        })
    }

    fn visit_forall(
        &mut self,
        vars: Vec<(Symbol, Sort)>,
        term: Self::T,
    ) -> Result<Self::T, Self::E> {
        Ok(Term::Forall {
            vars,
            term: Box::new(term),
        })
    }

    fn visit_exists(
        &mut self,
        vars: Vec<(Symbol, Sort)>,
        term: Self::T,
    ) -> Result<Self::T, Self::E> {
        Ok(Term::Exists {
            vars,
            term: Box::new(term),
        })
    }

    fn visit_match(
        &mut self,
        term: Self::T,
        cases: Vec<(Vec<Symbol>, Self::T)>,
    ) -> Result<Self::T, Self::E> {
        Ok(Term::Match {
            term: Box::new(term),
            cases,
        })
    }

    fn visit_attributes(
        &mut self,
        term: Self::T,
        attributes: Vec<(
            Keyword,
            crate::concrete::AttributeValue<Constant, Symbol, SExpr>,
        )>,
    ) -> Result<Self::T, Self::E> {
        Ok(Term::Attributes {
            term: Box::new(term),
            attributes,
        })
    }
}
