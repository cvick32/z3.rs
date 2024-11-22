use std::collections::{BTreeSet, HashMap};

use crate::{
    concrete::{Symbol, SyntaxBuilder},
    vmt::{variable::var_is_immutable, VARIABLE_FRAME_DELIMITER},
};

#[derive(Clone, Default)]
pub struct Instantiator {
    pub visitor: SyntaxBuilder,
    pub current_to_next_variables: HashMap<String, String>,
    pub frames: BTreeSet<usize>,
}

impl crate::rewriter::Rewriter for Instantiator {
    type V = SyntaxBuilder;
    type Error = crate::concrete::Error;

    fn visitor(&mut self) -> &mut Self::V {
        &mut self.visitor
    }

    fn process_symbol(&mut self, s: Symbol) -> Result<Symbol, Self::Error> {
        println!("sym: {}", s);
        let symbol_split = s.0.split(VARIABLE_FRAME_DELIMITER).collect::<Vec<_>>();
        if symbol_split.len() == 1 {
            // Symbol is not a variable
            Ok(s)
        } else {
            let (variable_name, time_str) = (symbol_split[0], symbol_split[1]);
            if var_is_immutable(variable_name) {
                return Ok(Symbol(format!("{}", variable_name)));
            }
            let time: usize = time_str.parse().unwrap();
            if &time == self.frames.first().unwrap() {
                Ok(Symbol(format!("{}", variable_name)))
            } else if &time == self.frames.last().unwrap() {
                match self.current_to_next_variables.get(variable_name) {
                    Some(_) => (),
                    None => println!("{:?}: {}", self.current_to_next_variables, variable_name),
                }
                Ok(Symbol(format!(
                    "{}",
                    self.current_to_next_variables.get(variable_name).unwrap()
                )))
            } else {
                todo!("Haven't implemented prophecy instantiation!")
            }
        }
    }
}
