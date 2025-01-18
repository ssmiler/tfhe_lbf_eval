use itertools::izip;

use crate::{
    lbf_circuit::{circuit::Node, Circuit},
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

impl FbsExecSeq {
    fn find_succ_cnt(circuit: &Circuit) -> HashMap<String, usize> {
        let mut succ_cnt: HashMap<String, usize> = Default::default();

        for node in circuit.nodes.iter() {
            match node {
                Node::Input { name } => {
                    succ_cnt.insert(name.to_owned(), 0);
                }
                Node::LinComb { inputs, output, .. } => {
                    for input in inputs {
                        *succ_cnt.get_mut(input).unwrap() += 1;
                    }
                    succ_cnt.insert(output.to_owned(), 0);
                }
                Node::Bootstrap { input, outputs, .. } => {
                    *succ_cnt.get_mut(input).unwrap() += 1;
                    for output in outputs {
                        succ_cnt.insert(output.to_owned(), 0);
                    }
                }
            }
        }

        succ_cnt
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
        let succ_cnt = Self::find_succ_cnt(circuit);

        inputs.into_iter().for_each(|(name, ct)| {
            let out_degree = *succ_cnt.get(&name).unwrap();
            ct_store.add(&name, ct, out_degree);
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
                    let cts = inputs.iter().map(|name| ct_store.get(name).unwrap());
                    let ct = server.lincomb(cts, coefs, *const_coef);

                    for input in inputs {
                        ct_store.unref(input);
                    }

                    let out_degree = *succ_cnt.get(output).unwrap();
                    ct_store.add(output, ct, out_degree);
                }
                Node::Bootstrap {
                    input,
                    outputs,
                    tables,
                } => {
                    for (out, table) in izip!(outputs, tables) {
                        let inp = ct_store.get(input).unwrap();
                        let ct = server.bootstrap(
                            inp.clone(),
                            &server.new_test_vector(table.clone()).unwrap(),
                        );

                        ct_store.unref(input);

                        let out_degree = *succ_cnt.get(out).unwrap();
                        ct_store.add(out, ct, out_degree);
                    }
                }
            }
        }

        for name in &circuit.outputs {
            let val = ct_store.get(name).unwrap();
            outputs.insert(name.clone(), val.clone());
        }

        Ok(outputs)
    }
}

#[derive(Default)]
struct CiphertextStore {
    data: HashMap<String, Ciphertext>,
    ref_cnt: HashMap<String, usize>,
}

impl CiphertextStore {
    fn add(&mut self, name: &str, ct: Ciphertext, nb_succ: usize) -> Option<Ciphertext> {
        self.ref_cnt.insert(name.to_owned(), nb_succ);
        self.data.insert(name.to_owned(), ct)
    }

    fn get(&self, name: &str) -> Option<&Ciphertext> {
        self.data.get(name)
    }

    fn unref(&mut self, name: &str) {
        let k = self.ref_cnt.get_mut(name).unwrap();
        *k -= 1;
        if *k == 0 {
            self.ref_cnt.remove(name);
            self.data.remove(name);
        }
    }
}
