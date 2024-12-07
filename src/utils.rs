use std::{
    io::{Error, Write},
    process::Command,
};

use smt2parser::vmt::smt::SMTProblem;
use tempfile::tempfile;

pub fn run_smtinterpol(smt_problem: SMTProblem) -> Result<i64, Error> {
    let interpolant_problem = smt_problem.to_smtinterpol();
    println!("{}", interpolant_problem);

    let mut temp_file = tempfile()?;
    writeln!(temp_file, "{interpolant_problem}")?;
    let interp_out = Command::new("java")
        .arg("-jar")
        .arg("./tools/smtinterpol-2.5-1386-gcca67e02.jar")
        .output()?;

    Ok(1)
}
