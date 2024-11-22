
use yardbird::{self, model_from_options, proof_loop, YardbirdOptions};

macro_rules! create_integration_test {
    ($test_name:ident, $example_name:literal, $num_instances:literal) => {
        #[test]
        fn $test_name() {
            let options = YardbirdOptions{
                filename: $example_name.into(),
                depth: 10,
                bmc_count: 2,
                print_vmt: false,
                run_benchmarks: false,
            };
            let mut vmt = model_from_options(&options);
            let mut used: Vec<String> = vec![];
            proof_loop(&options.depth, &mut vmt, &mut used);
            assert!(used.len() == $num_instances, "{} != {}", used.len(), $num_instances);
        }
    };
}

create_integration_test!(test_array_init_var, "examples/array_init_var.vmt", 4);
create_integration_test!(test_array_copy, "examples/array_copy.vmt", 5);
create_integration_test!(test_array_init_const, "examples/array_init_const.vmt", 12);