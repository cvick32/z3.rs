use egg::*;

define_language! {
    pub enum ArrayLanguage {
        Num(i64),
        "K" = ConstArr([Id; 1]),
        "Write" = Write([Id; 2]),
        "Read" = Read([Id; 2]),
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

fn not_equal(index_0: &'static str, index_1: &'static str) -> impl Fn(&mut EGraph<ArrayLanguage, ()>, Id, &Subst) -> bool {
    let var_0 = index_0.parse().unwrap();
    let var_1 = index_1.parse().unwrap();

    move |egraph, _, subst| egraph.find(subst[var_0]) != egraph.find(subst[var_1])
}