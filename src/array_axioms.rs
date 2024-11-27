use std::rc::Rc;

use egg::*;

use crate::{
    conflict_scheduler::ConflictScheduler,
    cost::BestVariableSubstitution,
    egg_utils::{DefaultCostFunction, Saturate},
};

define_language! {
    pub enum ArrayLanguage {
        Num(i64),
        "ConstArr-Int-Int" = ConstArr([Id; 1]),
        "Write-Int-Int" = Write([Id; 3]),
        "Read-Int-Int" = Read([Id; 2]),
        "and" = And(Box<[Id]>),
        "not" = Not(Id),
        "or" = Or(Box<[Id]>),
        "=>" = Implies([Id; 2]),
        "=" = Eq([Id; 2]),
        ">=" = Geq([Id; 2]),
        ">" = Gt([Id; 2]),
        "<=" = Leq([Id; 2]),
        "<" = Lt([Id; 2]),
        "+" = Plus([Id; 2]),
        "-" = Negate(Box<[Id]>),
        "*" = Times([Id; 2]),
        Symbol(Symbol),
    }
}

impl<N> Saturate for EGraph<ArrayLanguage, N>
where
    N: Analysis<ArrayLanguage> + Default + 'static,
{
    fn saturate(&mut self) -> Vec<String> {
        let egraph = std::mem::take(self);
        let scheduler = ConflictScheduler::new(BackoffScheduler::default());
        let instantiations = scheduler.instantiations();
        let mut runner = Runner::default()
            .with_egraph(egraph)
            .with_scheduler(scheduler)
            .run(&array_axioms());
        *self = std::mem::take(&mut runner.egraph);
        drop(runner);
        Rc::into_inner(instantiations).unwrap().into_inner()
    }
}

impl DefaultCostFunction for ArrayLanguage {
    fn cost_function() -> impl egg::CostFunction<ArrayLanguage> {
        BestVariableSubstitution
    }
}

fn array_axioms<N>() -> Vec<Rewrite<ArrayLanguage, N>>
where
    N: Analysis<ArrayLanguage> + 'static,
{
    vec![
        rewrite!("constant-array"; "(Read-Int-Int (ConstArr-Int-Int ?a) ?b)" => "?a"),
        rewrite!("read-after-write"; "(Read-Int-Int (Write-Int-Int ?a ?idx ?val) ?idx)" => "?val"),
        rewrite!("write-does-not-overwrite"; "(Read-Int-Int (Write-Int-Int ?a ?idx ?val) ?c)" => "(Read-Int-Int ?a ?c)" if not_equal("?idx", "?c")),
    ]
}

fn not_equal<N>(
    index_0: &'static str,
    index_1: &'static str,
) -> impl Fn(&mut EGraph<ArrayLanguage, N>, Id, &Subst) -> bool
where
    N: Analysis<ArrayLanguage>,
{
    let var_0 = index_0.parse().unwrap();
    let var_1 = index_1.parse().unwrap();

    move |egraph, _, subst| egraph.find(subst[var_0]) != egraph.find(subst[var_1])
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_conditional_axioms0() {
        let expr: RecExpr<ArrayLanguage> =
            "(Read-Int-Int (Write-Int-Int A 0 0) 1)".parse().unwrap();
        let runner = Runner::default()
            .with_expr(&expr)
            .run(&array_axioms::<()>());

        let gold: RecExpr<ArrayLanguage> = "(Read-Int-Int A 1)".parse().unwrap();
        assert!(runner.egraph.lookup_expr(&gold).is_some())
    }

    #[test]
    fn test_conditional_axioms1() {
        let expr: RecExpr<ArrayLanguage> =
            "(Read-Int-Int (Write-Int-Int A 0 0) 0)".parse().unwrap();
        let runner = Runner::default()
            .with_expr(&expr)
            .run(&array_axioms::<()>());

        let gold: RecExpr<ArrayLanguage> = "(Read-Int-Int A 0)".parse().unwrap();
        assert!(runner.egraph.lookup_expr(&gold).is_none())
    }

    /// Construct a sample model that is invalid according to the array axioms, and find
    /// an instantiation of an axiom that proves this.
    ///
    /// Let's take this sample model that is obviously invalid. We'll construct this by
    /// instantiating the terms `(Read-Int-Int (ConstArr-Int-Int 0) 0)` and `1` and unioning them in the
    /// egraph.
    ///
    /// ```
    /// (Read-Int-Int (ConstArr-Int-Int 0) 0) = 1
    /// ```
    ///
    /// Then I think that we want to get out an axiom instantiation that looks like
    /// `(Read-Int-Int (ConstArr-Int-Int 0) 0) = 0` because that will rule out that union being possible.
    #[test]
    fn invalid_const_array() {
        /* let mut egraph: EGraph<ArrayLanguage, _> = EGraph::new(()).with_explanations_enabled();

        let read_term: RecExpr<ArrayLanguage> =
            "(Read-Int-Int (ConstArr-Int-Int 0) 0)".parse().unwrap();
        let one_term: RecExpr<ArrayLanguage> = "1".parse().unwrap();

        let read_handle = egraph.add_expr(&read_term);
        let one_handle = egraph.add_expr(&one_term);

        egraph.union(read_handle, one_handle);
        egraph.saturate();

        let mut explanation =
            egraph.explain_equivalence(&"0".parse().unwrap(), &"1".parse().unwrap());

        // println!("{:#?}", explanation.explanation_trees);
        println!("{}", explanation.get_flat_string()); */
    }
}
