use std::collections::BTreeSet;

use crate::{
    concrete::{Symbol, SyntaxBuilder},
    vmt::{variable::var_is_immutable, VARIABLE_FRAME_DELIMITER},
};

/// This visits a Term and finds all of the frame numbers associated
/// with each variable in the Term.
/// For the Term (= a@0 a@1), we would expect FrameNumGetter.frame_nums to be {0, 1}.
/// We need this information in the Instantiator to when to plug in current variable
/// values or next variable values.
///
/// TODO: Using the Rewriter may not be the best choice here because it rebuilds the term.
/// But, using the TermVisitor like in LetExtract is more cumbersome.
#[derive(Clone, Default)]
pub struct FrameNumGetter {
    pub visitor: SyntaxBuilder,
    pub frame_nums: BTreeSet<usize>,
}

impl FrameNumGetter {
    pub fn new() -> Self {
        FrameNumGetter {
            visitor: SyntaxBuilder,
            frame_nums: BTreeSet::new(),
        }
    }
    
    pub(crate) fn max_min_difference(&self) -> usize {
        if self.frame_nums.len() < 2 {
            0
        } else {
            self.frame_nums.last().unwrap() - self.frame_nums.first().unwrap()
        }
    }
}

impl crate::rewriter::Rewriter for FrameNumGetter {
    type V = SyntaxBuilder;
    type Error = crate::concrete::Error;

    fn visitor(&mut self) -> &mut Self::V {
        &mut self.visitor
    }

    fn process_symbol(&mut self, s: Symbol) -> Result<Symbol, Self::Error> {
        let symbol_split = s.0.split(VARIABLE_FRAME_DELIMITER).collect::<Vec<_>>();
        if symbol_split.len() == 1 {
            // Symbol is not a variable
            Ok(s)
        } else {
            let (var_name, time_str) = (symbol_split[0], symbol_split[1]);
            if var_is_immutable(var_name) {
                // Don't add time step to frame_nums because immutable variables always
                // have the same value.
                Ok(s)
            } else {
                let time = time_str.parse().unwrap();
                self.frame_nums.insert(time);
                Ok(s)
            }
        }
    }
}
