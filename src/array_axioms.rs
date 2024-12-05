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
    fn cost_function() -> impl crate::egg_utils::CompareCost<Self> {
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

    move |egraph, _, subst| {
        panic!("in not equal");
        egraph.find(subst[var_0]) != egraph.find(subst[var_1])
    }
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

    #[test]
    fn test_conditional_axioms0_with_scheduluer() {
        let expr: RecExpr<ArrayLanguage> =
            "(Read-Int-Int (Write-Int-Int A 0 0) 1)".parse().unwrap();

        let scheduler = ConflictScheduler::new(BackoffScheduler::default());
        let instantiations = scheduler.instantiations();
        let runner = Runner::default()
            .with_expr(&expr)
            .with_scheduler(scheduler)
            .run(&array_axioms::<()>());

        println!("{:?}", instantiations.borrow().len());
        assert!(instantiations.borrow().len() == 1);
    }

    #[test]
    fn test_conditional_axioms1_with_scheduler() {
        let expr: RecExpr<ArrayLanguage> =
            "(Read-Int-Int (Write-Int-Int A 0 0) 0)".parse().unwrap();
        let runner = Runner::default()
            .with_expr(&expr)
            .run(&array_axioms::<()>());

        let gold: RecExpr<ArrayLanguage> = "(Read-Int-Int A 0)".parse().unwrap();
        assert!(runner.egraph.lookup_expr(&gold).is_none())
    }
}
