use std::{collections::HashMap, fs, path::PathBuf};

use clap::Parser;
use itertools::Itertools;
use lbf_eval::{
    circuit::parser::parse_lbf,
    executors::{clear::ClearExec, fbs::FbsExec},
    tfhe::gen_client_server,
};
use rand::{RngCore, SeedableRng};
use std::time::Instant;
use tfhe::shortint::prelude::*;

#[derive(Parser)]
#[command(name = "LBF executor", about = "Execute lbf file", long_about = None)]
struct Cli {
    /// Input LBF file
    #[arg(value_name = "FILE")]
    lbf_file: PathBuf,

    /// Number of execution threads
    #[arg(short, long, default_value_t = 1)]
    threads: usize,

    /// Seed
    #[arg(short, long, default_value_t = 42)]
    seed: u64,
}

fn build_random_inputs(inputs: &[String], seed: u64) -> HashMap<String, bool> {
    let mut rng = rand::prelude::StdRng::seed_from_u64(seed);
    inputs
        .iter()
        .map(|name| (name.clone(), rng.next_u32() % 2 == 1))
        .collect()
}

fn vals_to_string(names: &Vec<String>, vals: &HashMap<String, bool>) -> String {
    names
        .iter()
        .map(|name| format!("{} = {}", name, *vals.get(name).unwrap() as u8))
        .join(" ")
}

fn main() {
    let cli = Cli::parse();

    let tmp = fs::read_to_string(&cli.lbf_file).expect("Unable to read file");
    let circuit = parse_lbf(&tmp).expect("Cannot parse LBF file");

    println!("Read LBF file {:?}", cli.lbf_file);

    // generate random inputs
    let input_vals = build_random_inputs(&circuit.inputs, cli.seed);
    println!(
        "Input vals (randomly generated): {:?}",
        vals_to_string(&circuit.inputs, &input_vals)
    );

    let outputs_clear = ClearExec::new().eval(&circuit, &input_vals).unwrap();
    println!(
        "Outputs (clear exec): {}",
        vals_to_string(&circuit.outputs, &outputs_clear)
    );

    println!("FBS execution...");

    let start = Instant::now();
    let (client, server) = gen_client_server(PARAM_MESSAGE_1_CARRY_1_KS_PBS);
    let executor = FbsExec::new(server);
    let duration_keygen = start.elapsed();

    let start = Instant::now();
    let input_vals = input_vals
        .iter()
        .map(|(name, val)| (name.clone(), client.encrypt(*val as u8)))
        .collect();
    let duration_encrypt = start.elapsed();

    let start = Instant::now();
    let outputs = executor.eval(&circuit, input_vals).unwrap();
    let duration_exec = start.elapsed();

    let start = Instant::now();
    let outputs: HashMap<String, bool> = outputs
        .into_iter()
        .map(|(name, ct)| (name, client.decrypt(&ct) != 0))
        .collect();
    let duration_decrypt = start.elapsed();

    println!("Outputs: {}", vals_to_string(&circuit.outputs, &outputs));
    println!(
        "Timings:\n\tkeygen = {:.1?}\n\tencrypt = {:.1?}\n\texec = {:.1?}\n\tdecrypt = {:.1?}",
        duration_keygen, duration_encrypt, duration_exec, duration_decrypt
    );

    if outputs_clear != outputs {
        println!("Clear and FBS execution outputs do not match");
    }
}

fn mainf() {
    let s = r#".inputs a b c d
    .outputs e f g \
        h CONST1
    .lincomb CONST1     # CONSTANT
    1
    .lincomb a b \
     n1     # n1 = 2.a + b, n1 in {0, 1, 2, 3}, sq. norm2 = 2^2 + 1^2 = 5
    2 1
    .lincomb a b n2     # n2 = a - b + 1, n2 in {0, 1, 2}, sq. norm2 = 1^2 + 1^2 = 2
    1 -1 \
    1
    .bootstrap n1 e          # AND(a, b)
    000   \
    1
    .bootstrap n2 f          # XOR(a, b)
    101
    .bootstrap n2 g h        # 2-output bootstraping
    001                 # AND(a, NOT(b))
    0001                # XNOR(a, b)
    .end
    "#;

    let circuit = parse_lbf(s).unwrap();
    println!("Circuit: {:?}", circuit);

    let mut inputs = HashMap::<String, bool>::default();
    inputs.insert("a".to_string(), true);
    inputs.insert("b".to_string(), false);
    inputs.insert("c".to_string(), true);
    inputs.insert("d".to_string(), true);

    println!("Inputs: {:?}", inputs);
    {
        println!("Clear execution...");
        let clear_exec = ClearExec::new();
        let outputs = clear_exec.eval(&circuit, &inputs).unwrap();

        println!(
            "Outputs: {}",
            outputs
                .keys()
                .sorted()
                .map(|name| format!("{} = {}", name, outputs.get(name).unwrap()))
                .join(" ")
        );
    }

    {
        println!("FBS execution...");
        let (client, server) = gen_client_server(PARAM_MESSAGE_1_CARRY_1_KS_PBS);
        let executor = FbsExec::new(server);

        let inputs = inputs
            .iter()
            .map(|(name, val)| (name.clone(), client.encrypt(*val as u8)))
            .collect();
        let outputs = executor.eval(&circuit, inputs).unwrap();
        let outputs: HashMap<String, bool> = outputs
            .into_iter()
            .map(|(name, ct)| (name, client.decrypt(&ct) != 0))
            .collect();

        println!(
            "Outputs: {}",
            outputs
                .keys()
                .sorted()
                .map(|name| format!("{} = {}", name, outputs.get(name).unwrap()))
                .join(" ")
        );
    }
}
