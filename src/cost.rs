use egg::Language;
use smt2parser::vmt::VARIABLE_FRAME_DELIMITER;

use crate::array_axioms::ArrayLanguage;

/// Cost function describing how to extract terms from an eclass while we are
/// instantiating a rule violation with concrete terms.
pub struct BestVariableSubstitution;

impl egg::CostFunction<ArrayLanguage> for BestVariableSubstitution {
    type Cost = u32;

    fn cost<C>(&mut self, enode: &ArrayLanguage, mut costs: C) -> Self::Cost
    where
        C: FnMut(egg::Id) -> Self::Cost,
    {
        // TODO: fiddle with the cost function to get what we want.
        //       right now all I am doing is preferring everything else
        //       over Nums
        let op_cost = match enode {
            ArrayLanguage::Num(_) => 10,
            ArrayLanguage::ConstArr(_) => 1,
            ArrayLanguage::Write(_) => 1,
            ArrayLanguage::Read(_) => 1,
            ArrayLanguage::And(_) => 1,
            ArrayLanguage::Not(_) => 1,
            ArrayLanguage::Or(_) => 1,
            ArrayLanguage::Implies(_) => 1,
            ArrayLanguage::Eq(_) => 1,
            ArrayLanguage::Geq(_) => 1,
            ArrayLanguage::Gt(_) => 1,
            ArrayLanguage::Leq(_) => 1,
            ArrayLanguage::Lt(_) => 1,
            ArrayLanguage::Plus(_) => 1,
            ArrayLanguage::Negate(_) => 1,
            ArrayLanguage::Times(_) => 1,
            ArrayLanguage::Symbol(sym) => {
                if sym.as_str().contains(VARIABLE_FRAME_DELIMITER) {
                    1
                } else {
                    10
                }
            }
        };
        enode.fold(op_cost, |sum, id| sum + costs(id))
    }
}
