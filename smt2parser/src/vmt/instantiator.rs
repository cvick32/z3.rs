use std::collections::HashMap;

use crate::concrete::{Symbol, SyntaxBuilder};

#[derive(Clone)]
pub struct Instantiator {
    pub visitor: SyntaxBuilder,
    pub current_variables: Vec<String>,
    pub next_variables: HashMap<String, String>,
}

impl crate::rewriter::Rewriter for Instantiator {
    type V = SyntaxBuilder;
    type Error = crate::concrete::Error;

    fn visitor(&mut self) -> &mut Self::V {
        &mut self.visitor
    }

    fn process_symbol(&mut self, s: Symbol) -> Result<Symbol, Self::Error> {
        println!("sym: {}", s);
        let symbol_split = s.0.split("@").collect::<Vec<_>>();
        if symbol_split.len() == 1 {
            // Symbol is not a variable
            Ok(s)
        } else {
            let (variable_name, time) = (symbol_split[0], symbol_split[1]);
            if time == "1" {
                Ok(Symbol(format!("{}", variable_name)))
            } else if time == "2" {
                Ok(Symbol(format!(
                    "{}",
                    self.next_variables.get(variable_name).unwrap()
                )))
            } else {
                todo!("Haven't implemented better instantiation logic")
            }
        }
    }
}
