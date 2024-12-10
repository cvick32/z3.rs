use super::{NEXT_VARIABLE_NAME, VARIABLE_FRAME_DELIMITER};
use crate::concrete::{Symbol, SyntaxBuilder};

/// This is effectively the opposite of the BMCBuilder.
/// So we take a term that is currently using numbered
/// variables, like `a@1``, and rewrite to `a_next`, provided
/// that we are currently at step 0.
#[derive(Clone)]
pub struct NumberedToSymbolic {
    pub visitor: SyntaxBuilder,
    pub step: usize,
}

impl NumberedToSymbolic {
    pub fn add_step(&mut self) {
        self.step += 1;
    }
}

impl crate::rewriter::Rewriter for NumberedToSymbolic {
    type V = SyntaxBuilder;
    type Error = crate::concrete::Error;

    fn visitor(&mut self) -> &mut Self::V {
        &mut self.visitor
    }

    fn process_symbol(&mut self, s: Symbol) -> Result<Symbol, Self::Error> {
        if s.0.contains(VARIABLE_FRAME_DELIMITER) {
            let split: Vec<_> = s.0.split(VARIABLE_FRAME_DELIMITER).collect();
            let (var_name, frame_number): (&str, usize) = (split[0], split[1].parse().unwrap());
            if frame_number == self.step {
                Ok(Symbol(var_name.to_string()))
            } else if frame_number == self.step + 1 {
                Ok(Symbol(format!("{var_name}_{NEXT_VARIABLE_NAME}")))
            } else {
                panic!(
                    "Out of step variable frame {} for current step {}",
                    s.0, self.step
                );
            }
        } else {
            Ok(s)
        }
    }
}
