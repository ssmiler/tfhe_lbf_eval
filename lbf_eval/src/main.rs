use std::{collections::HashMap, fs, path::PathBuf};

use clap::Parser;
use itertools::Itertools;
use lbf_eval::{
    executors::{
        clear::ClearExec, fbs_exec::FbsExec, fbs_par::FbsExecPar, fbs_seq::FbsExecSeq,
        stats::CircuitStats,
    },
    lbf_circuit::parser::parse_lbf,
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

fn vals_to_string(names: &[String], vals: &HashMap<String, bool>) -> String {
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

    let stats = CircuitStats::new(&circuit);
    println!("Circuit stats {:?}", stats);

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
    let duration_keygen = start.elapsed();

    let executor: Box<dyn FbsExec> = if cli.threads == 1 {
        FbsExecSeq::new_boxed()
    } else {
        FbsExecPar::new_boxed()
    };

    let start = Instant::now();
    let input_vals = input_vals
        .iter()
        .map(|(name, val)| (name.clone(), client.encrypt(*val as u8)))
        .collect();
    let duration_encrypt = start.elapsed();

    let start = Instant::now();
    let outputs = executor.eval(server, &circuit, input_vals).unwrap();
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
