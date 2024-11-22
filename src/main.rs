use clap::Parser;
use std::{fs::File, io::Write};
use yardbird::{benchmark::run_benchmarks, model_from_options, proof_loop, YardbirdOptions};

fn main() {
    env_logger::init();
    let options = YardbirdOptions::parse();
    if options.run_benchmarks {
        run_benchmarks(&options);
    } else {
        let mut abstract_vmt_model = model_from_options(&options);
        let mut used_instances = vec![];
        proof_loop(&options.depth, &mut abstract_vmt_model, &mut used_instances);
        println!("NEEDED INSTANTIATIONS: {:#?}", used_instances);
        if options.print_vmt {
            let mut output = File::create("instantiated.vmt").unwrap();
            let _ = output.write(abstract_vmt_model.as_vmt_string().as_bytes());
        }
    }
}
