use std::{
    fs::File, io::{Error, Write}, process::Command
};
use smt2parser::vmt::smt::SMTProblem;

static INTERPOLANT_FILENAME: &str = "out.smt2";

pub fn run_smtinterpol(smt_problem: SMTProblem) -> Result<Vec<String>, Error> {
    let interpolant_problem = smt_problem.to_smtinterpol();
    println!("{}", interpolant_problem);
    let mut temp_file = File::create(INTERPOLANT_FILENAME)?;
    writeln!(temp_file, "{interpolant_problem}")?;
    let interp_out = Command::new("java")
        .arg("-jar")
        .arg("./tools/smtinterpol-2.5-1386-gcca67e02.jar")
        .arg("-w") // Only output interpolants.
        .arg(INTERPOLANT_FILENAME)
        .output()?;
    
    let string_stdout = String::from_utf8(interp_out.stdout).unwrap();
    let mut stdout = string_stdout.split("\n").map(|s| s.to_string()).collect::<Vec<_>>();
    assert_eq!(stdout[0], "unsat");

    // Split off at 1 as the 0th element should always be 'unsat' from
    // (check-sat).
    let interpolants = stdout.split_off(1);
    
    // TODO: parse interpolants 

    Ok(interpolants)
}
