use array_axioms::ArrayLanguage;
use clap::Parser;
use egg::RecExpr;
use smt2parser::{concrete::SyntaxBuilder, vmt::VMTModel, CommandStream};
use z3::{Config, Context, Solver};

mod array_axioms;
mod abstract_rewriter;

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Options {
    /// Name of the VMT file.
    #[arg(short, long)]
    filename: String,

    #[arg(short, long)]
    bmc_count: usize,

}


fn main() {
    let options = Options::parse();
    let file = std::io::BufReader::new(std::fs::File::open(options.filename.clone()).unwrap());
    let command_stream = CommandStream::new(file, SyntaxBuilder, Some(options.filename));
    let mut commands = vec![];
    for result in command_stream {
        match result {
            Ok(command) => commands.push(command),
            Err(_) => todo!(),
        }
    }
    let vmt_model = VMTModel::checked_from(commands).unwrap();
    let config: Config = Config::new();
    let context: Context = Context::new(&config);
    let solver = Solver::new(&context);

    for depth in 0..10 {
        let smt = vmt_model.unroll(depth);
        solver.push();
        solver.from_string(smt.to_smtlib2());
        let _ = solver.check();
        let model = solver.get_model().unwrap();
        let array_language_expr = get_array_language_expression(model);
        let expr: RecExpr<ArrayLanguage> = array_language_expr.parse().unwrap();

    }


}

fn get_array_language_expression(model: z3::Model<'_>) -> String {
    // Put all equalities in the model and saturate. 
    // Since f
    todo!()
}
