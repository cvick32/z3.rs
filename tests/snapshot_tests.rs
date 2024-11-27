use insta::assert_debug_snapshot;
use smt2parser::vmt::VMTModel;
use std::{
    fs::read_dir,
    path::Path,
    sync::mpsc::{self, RecvTimeoutError},
    thread,
    time::Duration,
};
use yardbird::proof_loop;

#[derive(Debug)]
enum BenchStatus {
    Good,
    Timeout,
    Panic,
}

fn run_with_timeout<F, T>(f: F, timeout: Duration) -> (BenchStatus, T)
where
    F: FnOnce() -> T + Send + 'static,
    T: Send + Default + 'static,
{
    let (tx, rx) = mpsc::channel();
    let _ = thread::spawn(move || {
        let result = f();
        if let Ok(()) = tx.send(result) {}
    });

    match rx.recv_timeout(timeout) {
        Ok(insts) => (BenchStatus::Good, insts),
        Err(RecvTimeoutError::Timeout) => (BenchStatus::Timeout, T::default()),
        Err(RecvTimeoutError::Disconnected) => (BenchStatus::Panic, T::default()),
    }
}

#[allow(unused)]
#[derive(Debug)]
struct BenchmarkResult {
    example_name: String,
    status: BenchStatus,
    used_instantiations: Vec<String>,
}

fn run_benchmark(filename: impl AsRef<Path>) -> BenchmarkResult {
    let conrete_model = VMTModel::from_path(filename.as_ref()).unwrap();
    let mut abstract_model = conrete_model.abstract_array_theory();
    let (status, used_instantiations) = run_with_timeout(
        move || {
            let mut used_instantiations = vec![];
            proof_loop(&10_u8, &mut abstract_model, &mut used_instantiations).unwrap();
            used_instantiations
        },
        Duration::from_secs(20),
    );
    BenchmarkResult {
        example_name: filename.as_ref().to_string_lossy().to_string(),
        status,
        used_instantiations,
    }
}

#[test]
fn test_examples() {
    for entry in read_dir("./examples/").unwrap() {
        let path = entry.unwrap().path();
        let name = path.file_name().unwrap().to_str().unwrap();
        assert_debug_snapshot!(name, run_benchmark(&path));
    }
}
