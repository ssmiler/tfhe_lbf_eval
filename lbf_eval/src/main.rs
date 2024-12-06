use std::collections::HashMap;

use itertools::Itertools;
use lbf_eval::{
    circuit::parser::parse_lbf,
    executors::{clear::ClearExec, fbs::FbsExec},
    tfhe::gen_client_server,
};
use tfhe::shortint::prelude::*;

fn main() {
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
