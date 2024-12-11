use std::{fs::File, io::Write};

use crate::analysis::SaturationInequalities;
use anyhow::anyhow;
use array_axioms::ArrayLanguage;
use clap::Parser;
use egg_utils::Saturate;
use log::{debug, info};
use smt2parser::vmt::VMTModel;
use utils::run_smtinterpol;
use z3::{Config, Context, Solver};

pub mod analysis;
pub mod array_axioms;
pub mod benchmark;
pub mod conflict_scheduler;
mod cost;
mod egg_utils;
mod interpolant;
pub mod logger;
mod utils;

#[derive(Parser, Debug, Clone)]
#[command(version, about, long_about = None)]
pub struct YardbirdOptions {
    /// Name of the VMT file.
    #[arg(short, long)]
    pub filename: String,

    /// BMC depth until quitting.
    #[arg(short, long, default_value_t = 10)]
    pub depth: u8,

    /// How many times BMC should be UNSAT until we check with an invariant generator.
    #[arg(short, long, default_value_t = 1)]
    pub bmc_count: usize,

    /// Output VMT files before and after instantiation.
    #[arg(short, long, default_value_t = false)]
    pub print_vmt: bool,

    /// Run SMTInterpol when BMC depth is UNSAT
    #[arg(short, long, default_value_t = false)]
    pub interpolate: bool,
}

#[derive(Debug)]
pub struct ProofLoopResult {
    pub model: VMTModel,
    pub used_instances: Vec<String>,
    pub const_instances: Vec<String>,
}

/// The main verification loop.
pub fn proof_loop(options: &YardbirdOptions) -> anyhow::Result<ProofLoopResult> {
    let mut abstract_vmt_model = model_from_options(options);
    let mut used_instances = vec![];
    let mut const_instances = vec![];
    let config: Config = Config::new();
    let context: Context = Context::new(&config);
    for depth in 0..options.depth {
        info!("STARTING BMC FOR DEPTH {}", depth);
        for _ in 0..10 {
            // Run max of 10 iterations for depth
            // Currently run once, this will eventually run until UNSAT
            let smt = abstract_vmt_model.unroll(depth);
            let solver = Solver::new(&context);
            solver.from_string(smt.to_bmc());
            debug!("smt2lib program:\n{}", smt.to_bmc());
            // TODO: abstract this out somehow
            let mut egraph: egg::EGraph<ArrayLanguage, _> =
                egg::EGraph::new(SaturationInequalities).with_explanations_enabled();
            for term in smt.get_assert_terms() {
                egraph.add_expr(&term.parse()?);
            }
            match solver.check() {
                z3::SatResult::Unsat => {
                    info!("RULED OUT ALL COUNTEREXAMPLES OF DEPTH {}", depth);
                    if options.interpolate {
                        let interpolants = run_smtinterpol(smt);
                        match interpolants {
                            Ok(_interps) => (),
                            Err(err) => println!("Error when computing interpolants: {err}"),
                        }
                    }
                    break;
                }
                z3::SatResult::Unknown => {
                    // CV: I've seen Z3 return unknown then re-run Z3 and gotten SAT or UNSAT.
                    // This might be a place to retry at least once before panicking.
                    panic!("Z3 RETURNED UNKNOWN!");
                }
                z3::SatResult::Sat => {
                    // find Array theory fact that rules out counterexample
                    let model = solver.get_model().ok_or(anyhow!("No z3 model"))?;
                    debug!("model:\n{}", model);

                    for func_decl in model.iter() {
                        if func_decl.arity() == 0 {
                            // VARIABLE
                            // Apply no arguments to the constant so we can call get_const_interp.
                            let func_decl_ast = func_decl.apply(&[]);
                            let var_id = egraph.add_expr(&func_decl.name().parse()?);
                            let value = model
                                .get_const_interp(&func_decl_ast)
                                .expect("Model failure.");
                            let value_id = egraph.add_expr(&value.to_string().parse()?);
                            egraph.union(var_id, value_id);
                        } else {
                            // FUNCTION DEF
                            let interpretation = model
                                .get_func_interp(&func_decl)
                                .ok_or(anyhow!("No func interp"))?;
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
                                let function_id = egraph.add_expr(&function_call.parse()?);
                                let value_id =
                                    egraph.add_expr(&entry.get_value().to_string().parse()?);
                                egraph.union(function_id, value_id);
                            }
                        }
                    }
                    egraph.rebuild();
                    let (instantiations, const_instantiations) = egraph.saturate();
                    const_instances.extend_from_slice(&const_instantiations);

                    // add all instantiations to the model,
                    // if we have already seen all instantiations, break
                    // TODO: not sure if this is correct...
                    let no_progress = instantiations.into_iter().all(|inst| {
                        !abstract_vmt_model.add_instantiation(inst, &mut used_instances)
                    });
                    if no_progress {
                        return Err(anyhow!("Failed to add new instantations"));
                    }
                }
            }
        }
    }
    Ok(ProofLoopResult {
        model: abstract_vmt_model,
        used_instances,
        const_instances,
    })
}

pub fn model_from_options(options: &YardbirdOptions) -> VMTModel {
    let concrete_vmt_model = VMTModel::from_path(&options.filename).unwrap();
    let abstract_vmt_model = concrete_vmt_model.abstract_array_theory();
    if options.print_vmt {
        let mut output = File::create("original.vmt").unwrap();
        let _ = output.write(abstract_vmt_model.as_vmt_string().as_bytes());
    }
    abstract_vmt_model
}
