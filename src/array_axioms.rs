use egg::*;

define_language! {
    pub enum ArrayLanguage {
        Num(i64),
        "ConstArr-Int-Int" = ConstArr([Id; 1]),
        "Write-Int-Int" = Write([Id; 3]),
        "Read-Int-Int" = Read([Id; 2]),
        "and" = And(Box<[Id]>),
        "not" = Not(Id),
        "=" = Eq([Id; 2]),
        ">=" = Geq([Id; 2]),
        Symbol(Symbol),
    }
}

pub fn make_array_axioms() -> Vec<Rewrite<ArrayLanguage, ()>> {
    vec![
        rewrite!("constant-array"; "(Read (K ?a) ?b)" => "?a"),
        rewrite!("read-after-write"; "(Read (Write ?a ?idx ?val) ?idx)" => "?val"),
        rewrite!("write-does-not-overwrite"; "(Read (Write ?a ?idx ?val) ?c)" => "(Read ?a ?c)" if not_equal("?idx", "?c")),
    ]
}

fn not_equal(
    index_0: &'static str,
    index_1: &'static str,
) -> impl Fn(&mut EGraph<ArrayLanguage, ()>, Id, &Subst) -> bool {
    let var_0 = index_0.parse().unwrap();
    let var_1 = index_1.parse().unwrap();

    move |egraph, _, subst| egraph.find(subst[var_0]) != egraph.find(subst[var_1])
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_conditional_axioms0() {
        let expr: RecExpr<ArrayLanguage> = "(Read (Write A 0 0) 1)".parse().unwrap();
        let runner = Runner::default().with_expr(&expr).run(&make_array_axioms());

        let gold: RecExpr<ArrayLanguage> = "(Read A 1)".parse().unwrap();
        assert!(runner.egraph.lookup_expr(&gold).is_some())
    }

    #[test]
    fn test_conditional_axioms1() {
        let expr: RecExpr<ArrayLanguage> = "(Read (Write A 0 0) 0)".parse().unwrap();
        let runner = Runner::default().with_expr(&expr).run(&make_array_axioms());

        let gold: RecExpr<ArrayLanguage> = "(Read A 0)".parse().unwrap();
        assert!(runner.egraph.lookup_expr(&gold).is_none())
    }
}
