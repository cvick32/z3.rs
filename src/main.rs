use clap::Parser;
use smt2parser::{get_commands, vmt::VMTModel};
use z3::{Config, Context, Solver};

mod array_axioms;

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Options {
    /// Name of the VMT file.
    #[arg(short, long)]
    filename: String,

    // How many times BMC should be UNSAT until we check with an invariant generator.
    #[arg(short, long, default_value_t = 1)]
    bmc_count: usize,

}

fn main() {
    let options = Options::parse();
    let content = std::io::BufReader::new(std::fs::File::open(options.filename.clone()).unwrap());
    let commands = get_commands(content, options.filename);
    let concrete_vmt_model = VMTModel::checked_from(commands).unwrap();
    let abstract_vmt_model = concrete_vmt_model.abstract_array_theory();
    let config: Config = Config::new();
    let context: Context = Context::new(&config);
    for depth in 0..10 {
        for d in 0..1 { // Currently run once, this will eventually run until UNSAT
            let smt = abstract_vmt_model.unroll(depth);
            let solver = Solver::new(&context);
            solver.from_string(smt.to_smtlib2());
            match solver.check() {
                z3::SatResult::Unsat => break, // Ruled out all counterexamples of this depth.
                z3::SatResult::Unknown => todo!(),
                z3::SatResult::Sat => {
                    // find Array theory fact that rules out counterexample
                    let model = solver.get_model().unwrap();
                    println!("{}", model);
                    // Model to Egraph
                    // Find one or many theory violations
                    // Add violations as facts
                },
            }
            
        }
    }
}
