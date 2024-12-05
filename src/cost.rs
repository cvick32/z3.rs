use egg::Language;
use smt2parser::vmt::VARIABLE_FRAME_DELIMITER;

use crate::{array_axioms::ArrayLanguage, egg_utils::CompareCost};

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
            ArrayLanguage::ConstArr(_) => 0,
            ArrayLanguage::Write(_) => 0,
            ArrayLanguage::Read(_) => 0,
            ArrayLanguage::And(_) => 0,
            ArrayLanguage::Not(_) => 0,
            ArrayLanguage::Or(_) => 0,
            ArrayLanguage::Implies(_) => 0,
            ArrayLanguage::Eq(_) => 0,
            ArrayLanguage::Geq(_) => 0,
            ArrayLanguage::Gt(_) => 0,
            ArrayLanguage::Leq(_) => 0,
            ArrayLanguage::Lt(_) => 0,
            ArrayLanguage::Plus(_) => 0,
            ArrayLanguage::Negate(_) => 0,
            ArrayLanguage::Times(_) => 0,
            ArrayLanguage::Symbol(sym) => {
                if sym.as_str().contains(VARIABLE_FRAME_DELIMITER) {
                    0
                } else {
                    // TODO: extend language to uninterpreted sort constants to 
                    // constants instead of symbols. 
                    // Ex: Array-Int-Int!val!0 is currently a symbol when it should be a
                    // constant.
                    10
                }
            }
        };
        enode.fold(op_cost, |sum, id| sum + costs(id))
    }
}

impl CompareCost<ArrayLanguage> for BestVariableSubstitution {
    fn lt(&self, x: Self::Cost, y: u32) -> bool {
        x < y
    }
}