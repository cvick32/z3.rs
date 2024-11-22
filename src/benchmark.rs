use std::{
    fs::{read_dir, File}, io::Write, sync::mpsc::{self, RecvTimeoutError}, thread, time::Duration
};

use serde::Serialize;

use crate::{model_from_options, proof_loop, YardbirdOptions};

#[derive(Debug, Serialize)]
enum BenchResult {
    Good,
    Timeout,
    Panic,
}

fn run_with_timeout<F, T>(f: F, timeout: Duration) -> BenchResult
where
    F: FnOnce() -> T + Send + 'static,
    T: Send + 'static,
{
    let (tx, rx) = mpsc::channel();
    let _ = thread::spawn(move || {
        let result = f();
        match tx.send(result) {
            Ok(()) => {} // everything good
            Err(_) => {} // we have been released, don't panic
        }
    });

    match rx.recv_timeout(timeout) {
        Ok(_) => BenchResult::Good,
        Err(RecvTimeoutError::Timeout) => BenchResult::Timeout,
        Err(RecvTimeoutError::Disconnected) => BenchResult::Panic,
    }
}

#[derive(Debug, Serialize)]
struct BenchmarkResult {
    example_name: String,
    result: BenchResult,
}
pub fn run_benchmarks(options: &YardbirdOptions) {
    let mut bench_results = vec![];
    for path in read_dir("./examples/").unwrap() {
        let path_string: String = path.unwrap().path().to_str().unwrap().to_string();
        if path_string.contains("2dim") {
            println!("Skipping: {}", path_string);
            continue;
        }
        println!("Trying: {}", path_string);
        let new_options = YardbirdOptions {
            filename: path_string.clone(),
            depth: options.depth,
            bmc_count: options.bmc_count,
            print_vmt: options.print_vmt,
            run_benchmarks: options.run_benchmarks,
        };
        let mut abstract_vmt_model = model_from_options(&new_options);
        let mut used_instances = vec![];
        let result = run_with_timeout(
            move || {
                proof_loop(&new_options.depth, &mut abstract_vmt_model, &mut used_instances);
            },
            Duration::from_secs(10),
        );
        bench_results.push(BenchmarkResult {
            example_name: path_string.clone(),
            result,
        });
    }
    println!("{:?}", bench_results);
    let mut output = File::create("benchmark-results.json").unwrap();
    let _ = output.write(serde_json::to_string(&bench_results).unwrap().as_bytes());
    println!("Tried {} benchmarks.", bench_results.len());
}
