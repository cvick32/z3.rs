use array_axioms::ArrayLanguage;
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

    let mut egraph: egg::EGraph<ArrayLanguage, ()> = egg::EGraph::new(());

    for depth in 0..10 {
        println!("STARTING BMC FOR DEPTH {}", depth);
        for _d in 0..1 {
            // Currently run once, this will eventually run until UNSAT
            let smt = abstract_vmt_model.unroll(depth);
            let solver = Solver::new(&context);
            solver.from_string(smt.to_smtlib2());
            println!("{:#?}", smt.get_assert_terms());

            for term in smt.get_assert_terms() {
                println!("{term}");
                egraph.add_expr(&term.parse().unwrap());
            }

            println!("{:?}", egraph.dump());
            egraph.dot().to_pdf("babies_first_egraph.pdf").unwrap();

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
                    todo!("{}", model);
                    // Model to Egraph
                    // Find one or many theory violations
                    // Add violations as facts
                }
            }
        }
    }
}
