use egg::{Analysis, DidMerge, EGraph, Id, Justification};

use crate::array_axioms::ArrayLanguage;

#[derive(Default)]
pub struct SaturationInequalities {}
impl Analysis<ArrayLanguage> for SaturationInequalities {
    type Data = bool;
    fn make(egraph: &EGraph<ArrayLanguage, Self>, enode: &ArrayLanguage) -> Self::Data {
        false
    }
    fn merge(&mut self, a: &mut Self::Data, b: Self::Data) -> DidMerge {
        *a = true;
        DidMerge(false, true)
    }

    fn pre_union(
        egraph: &EGraph<ArrayLanguage, Self>,
        id1: Id,
        id2: Id,
        justification: &Option<Justification>,
    ) {
    }

    fn modify(egraph: &mut EGraph<ArrayLanguage, Self>, id: Id) {}
}
