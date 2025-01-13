use std::collections::HashMap;

use itertools::izip;

use crate::circuit::{circuit::Node, Circuit};

pub struct ClearExec;

impl ClearExec {
    pub fn new() -> ClearExec {
        ClearExec
    }

    pub fn eval(
        &self,
        circuit: &Circuit,
        inputs: &HashMap<String, bool>,
    ) -> Result<HashMap<String, bool>, String> {
        let mut node_val = HashMap::<String, u8>::new();
        let mut outputs = HashMap::<String, bool>::new();

        for node in &circuit.nodes {
            match node {
                Node::Input { name } => {
                    let val = *inputs
                        .get(name)
                        .ok_or(format!("Cannot find input {}", name))?;
                    node_val.insert(name.clone(), val as u8);
                }
                Node::LinComb {
                    inputs,
                    output,
                    coefs,
                    const_coef,
                } => {
                    let mut res = *const_coef;
                    for (inp, coef) in izip!(inputs, coefs) {
                        let val = *node_val
                            .get(inp)
                            .ok_or(format!("Cannot find node {}", inp))?;
                        res += val as i8 * *coef;
                    }
                    if res < 0 {
                        return Err(format!("Lincomb {} output is negative", output));
                    }
                    node_val.insert(output.clone(), res as u8);
                }
                Node::Bootstrap {
                    input,
                    outputs,
                    tables,
                } => {
                    let val = *node_val
                        .get(input)
                        .ok_or(format!("Cannot find node {}", input))?;
                    for (out, table) in izip!(outputs, tables) {
                        let val = val as usize;
                        if val > table.len() {
                            return Err(format!("Bootstrap table {:?} is too short ", table));
                        }
                        let res = table[val];
                        node_val.insert(out.clone(), res as u8);
                    }
                }
            }
        }

        for name in &circuit.outputs {
            let val = *node_val
                .get(name)
                .ok_or(format!("Cannot find node {}", name))?;
            if (val != 0) & (val != 1) {
                return Err(format!(
                    "Node {} value is not boolean {}, missing bootstrapping?",
                    name, val
                ));
            }
            outputs.insert(name.clone(), val != 0);
        }

        Ok(outputs)
    }
}
