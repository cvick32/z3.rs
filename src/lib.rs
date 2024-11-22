use std::{fs::File, io::Write};

use crate::analysis::SaturationInequalities;
use array_axioms::{ArrayLanguage, Saturate};
use clap::Parser;
use log::debug;
use smt2parser::{get_commands, vmt::VMTModel};
use z3::{Config, Context, Solver};

pub mod analysis;
pub mod array_axioms;
pub mod benchmark;
pub mod conflict_scheduler;

#[derive(Parser, Debug, Clone)]
#[command(version, about, long_about = None)]
pub struct YardbirdOptions {
    /// Name of the VMT file.
    #[arg(short, long)]
    pub filename: String,

    // BMC depth until quitting.
    #[arg(short, long, default_value_t = 10)]
    pub depth: u8,

    // How many times BMC should be UNSAT until we check with an invariant generator.
    #[arg(short, long, default_value_t = 1)]
    pub bmc_count: usize,

    // Output VMT files before and after instantiation.
    #[arg(short, long, default_value_t = false)]
    pub print_vmt: bool,

    // Run all of the benchmarks. 
    #[arg(short, long, default_value_t = false)]
    pub run_benchmarks: bool,
}

/// The main verification loop. 
pub fn proof_loop(bmc_depth: &u8, vmt_model: &mut VMTModel, used_instances: &mut Vec<String>) {
    let config: Config = Config::new();
    let context: Context = Context::new(&config);
    for depth in 0..*bmc_depth {
        println!("STARTING BMC FOR DEPTH {}", depth);
        for _ in 0..10 { // Run max of 10 iterations for depth
            // Currently run once, this will eventually run until UNSAT
            let smt = vmt_model.unroll(depth);
            let solver = Solver::new(&context);
            solver.from_string(smt.to_smtlib2());
            debug!("{}", solver);
            let mut egraph: egg::EGraph<ArrayLanguage, _> =
                egg::EGraph::new(SaturationInequalities {}).with_explanations_enabled();
            for term in smt.get_assert_terms() {
                egraph.add_expr(&term.parse().unwrap());
            }
            match solver.check() {
                z3::SatResult::Unsat => {
                    println!("RULED OUT ALL COUNTEREXAMPLES OF DEPTH {}", depth);
                    break;
                }
                z3::SatResult::Unknown => {
                    // CV: I've seen Z3 return unknown then re-run Z3 and gotten SAT or UNSAT.
                    // This might be a place to retry at least once before panicking.
                    panic!("Z3 RETURNED UNKNOWN!");
                }
                z3::SatResult::Sat => {
                    // find Array theory fact that rules out counterexample
                    let model = solver.get_model().unwrap();
                    debug!("{}", model);

                    for func_decl in model.iter() {
                        if func_decl.arity() == 0 {
                            // VARIABLE
                            let func_decl_ast = func_decl.apply(&[]); // Apply no arguments to the constant so we can call get_const_interp.
                            let var_id = egraph.add_expr(&func_decl.name().parse().unwrap());
                            let value = model
                                .get_const_interp(&func_decl_ast)
                                .expect("Model failure.");
                            let value_id = egraph.add_expr(&value.to_string().parse().unwrap());
                            egraph.union(var_id, value_id);
                        } else {
                            // FUNCTION DEF
                            let interpretation = model.get_func_interp(&func_decl).unwrap();
                            for entry in interpretation.get_entries() {
                                let function_call = format!(
                                    "({} {})",
                                    func_decl.name(),
                                    entry
                                        .get_args()
                                        .iter()
                                        .map(ToString::to_string)
                                        .collect::<Vec<_>>()
                                        .join(" ")
                                );
                                let function_id = egraph.add_expr(&function_call.parse().unwrap());
                                let value_id = egraph
                                    .add_expr(&entry.get_value().to_string().parse().unwrap());
                                egraph.union(function_id, value_id);
                            }
                        }
                    }
                    egraph.rebuild();
                    //egraph.dot().to_pdf("unsaturated.pdf").unwrap();
                    let instantiations = egraph.saturate();
                    //egraph.dot().to_pdf("saturated.pdf").unwrap();
                    //println!("{:?}", egraph.dump());
                    for inst in instantiations {
                        // Adds the used instances.
                        vmt_model.add_instantiation(inst, used_instances);
                    }
                }
            }
        }
    }
    println!("USED INSTANCES: {:#?}", used_instances);
}

pub fn model_from_options(options: &YardbirdOptions) -> VMTModel {
    let content = std::io::BufReader::new(std::fs::File::open(options.filename.clone()).unwrap());
    let commands = get_commands(content, options.filename.clone());
    let concrete_vmt_model = VMTModel::checked_from(commands).unwrap();
    let abstract_vmt_model = concrete_vmt_model.abstract_array_theory();
    if options.print_vmt {
        let mut output = File::create("original.vmt").unwrap();
        let _ = output.write(abstract_vmt_model.as_vmt_string().as_bytes());
    }
    abstract_vmt_model
}
