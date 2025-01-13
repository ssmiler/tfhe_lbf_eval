use itertools::izip;

use crate::{
    circuit::{circuit::Node, Circuit},
    tfhe::{Ciphertext, Server},
};
use std::collections::HashMap;

use super::fbs_exec::FbsExec;

#[derive(Default)]
pub struct FbsExecSeq;

impl FbsExecSeq {
    pub fn new() -> Self {
        Self
    }

    pub fn new_boxed() -> Box<Self> {
        Box::new(Self)
    }
}

impl FbsExec for FbsExecSeq {
    fn eval(
        &self,
        server: Server,
        circuit: &Circuit,
        inputs: HashMap<String, Ciphertext>,
    ) -> Result<HashMap<String, Ciphertext>, String> {
        let mut outputs = HashMap::<String, Ciphertext>::new();

        let mut ct_store = CiphertextStore::default();
        inputs.into_iter().for_each(|(name, ct)| {
            ct_store.add(&name, ct);
        });

        for node in &circuit.nodes {
            match node {
                Node::Input { .. } => {}
                Node::LinComb {
                    inputs,
                    output,
                    coefs,
                    const_coef,
                } => {
                    let cts = inputs.iter().map(|name| ct_store.get(name));
                    let ct = server.lincomb(cts, coefs, *const_coef);
                    ct_store.add(output, ct)
                }
                Node::Bootstrap {
                    input,
                    outputs,
                    tables,
                } => {
                    for (out, table) in izip!(outputs, tables) {
                        let inp = ct_store.get(input);
                        let ct = server.bootstrap(
                            inp.clone(),
                            &server.new_test_vector(table.clone()).unwrap(),
                        );
                        ct_store.add(out, ct);
                    }
                }
            }
        }

        for name in &circuit.outputs {
            let val = ct_store.get(&name);
            outputs.insert(name.clone(), val.clone());
        }

        Ok(outputs)
    }
}

#[derive(Default)]
struct CiphertextStore {
    arena: Vec<Ciphertext>,
    ct_idx: HashMap<String, usize>,
}

impl CiphertextStore {
    fn add(&mut self, name: &String, ct: Ciphertext) {
        let idx = self.arena.len();
        self.arena.push(ct);
        match self.ct_idx.insert(name.clone(), idx) {
            Some(_) => unreachable!(),
            None => (),
        };
    }

    fn get(&self, name: &String) -> &Ciphertext {
        let idx = match self.ct_idx.get(name) {
            Some(idx) => *idx,
            None => unreachable!(),
        };
        &self.arena[idx]
    }
}
