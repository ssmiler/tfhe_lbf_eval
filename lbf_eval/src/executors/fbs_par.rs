use std::{collections::HashMap, sync::Arc};

use ::futures::future::try_join_all;
use itertools::izip;
use tokio::{
    runtime::{Builder, Runtime},
    sync::broadcast::{channel, Receiver, Sender},
};

use crate::{
    lbf_circuit::{circuit, Circuit},
    tfhe::{Ciphertext, Server},
};

use super::fbs_exec::FbsExec;

enum NodeExec {
    Input {
        ct: Ciphertext,
        output: Sender<Ciphertext>,
    },
    LinComb {
        server: Arc<Server>,
        output: Sender<Ciphertext>,
        inputs: Vec<Receiver<Ciphertext>>,
        coefs: Vec<i8>,
        const_coef: i8,
    },
    Bootstrap {
        server: Arc<Server>,
        output: Sender<Ciphertext>,
        input: Receiver<Ciphertext>,
        table: Vec<bool>,
    },
}

impl NodeExec {
    pub fn new_input(ct: Ciphertext) -> NodeExec {
        let (output, _) = channel::<Ciphertext>(1);
        NodeExec::Input { ct, output }
    }

    pub fn new_lincomb(
        server: Arc<Server>,
        inputs: Vec<Receiver<Ciphertext>>,
        coefs: Vec<i8>,
        const_coef: i8,
    ) -> Self {
        let (output, _) = channel::<Ciphertext>(1);
        NodeExec::LinComb {
            server,
            output,
            inputs,
            coefs,
            const_coef,
        }
    }
    pub fn new_bootstrap(
        server: Arc<Server>,
        input: Receiver<Ciphertext>,
        table: Vec<bool>,
    ) -> Self {
        let (output, _) = channel::<Ciphertext>(1);
        NodeExec::Bootstrap {
            server,
            output,
            input,
            table,
        }
    }
}

impl NodeExec {
    fn get_output(&self) -> Receiver<Ciphertext> {
        match self {
            NodeExec::Input { output, .. }
            | NodeExec::LinComb { output, .. }
            | NodeExec::Bootstrap { output, .. } => output.subscribe(),
        }
    }
    async fn run(self) -> Result<usize, String> {
        match self {
            NodeExec::Input { ct, output } => output.send(ct).map_err(|e| e.to_string()),
            NodeExec::LinComb {
                server,
                output,
                mut inputs,
                coefs,
                const_coef,
            } => {
                let inp_vals = try_join_all(inputs.iter_mut().map(|input| input.recv()))
                    .await
                    .map_err(|e| e.to_string())?;

                let val = server.lincomb(&inp_vals, &coefs, const_coef);
                output.send(val).map_err(|e| e.to_string())
            }
            NodeExec::Bootstrap {
                server,
                output,
                mut input,
                table,
            } => {
                let inp = input.recv().await.map_err(|e| e.to_string())?;
                let val = server.bootstrap(inp, &server.new_test_vector(table.clone()).unwrap());

                output.send(val).map_err(|e| e.to_string())
            }
        }
    }
}

pub struct FbsExecPar {
    rt: Runtime,
}

impl Default for FbsExecPar {
    fn default() -> Self {
        Self::new(1)
    }
}

impl FbsExecPar {
    pub fn new(nb_threads: usize) -> Self {
        Self {
            rt: Builder::new_multi_thread()
                .worker_threads(nb_threads)
                .build()
                .unwrap(),
        }
    }

    pub fn new_boxed(nb_threads: usize) -> Box<Self> {
        Box::new(Self::new(nb_threads))
    }
}

impl FbsExec for FbsExecPar {
    fn eval(
        &self,
        server: Server,
        circuit: &Circuit,
        mut inputs: HashMap<String, Ciphertext>,
    ) -> Result<HashMap<String, Ciphertext>, String> {
        let mut nodes: HashMap<String, NodeExec> = Default::default();
        let server = Arc::new(server);

        for node in &circuit.nodes {
            match node.clone() {
                circuit::Node::Input { name } => {
                    let ct = inputs
                        .remove(&name)
                        .ok_or(format!("Cannot find input {}", name))?;
                    let node = NodeExec::new_input(ct);
                    nodes.insert(name, node);
                }
                circuit::Node::LinComb {
                    inputs,
                    output,
                    coefs,
                    const_coef,
                } => {
                    let inputs = inputs
                        .into_iter()
                        .map(|name| nodes.get(&name).unwrap().get_output())
                        .collect();
                    let node = NodeExec::new_lincomb(server.clone(), inputs, coefs, const_coef);
                    nodes.insert(output, node);
                }
                circuit::Node::Bootstrap {
                    input,
                    outputs,
                    tables,
                } => {
                    for (output, table) in izip!(outputs, tables) {
                        let input = nodes.get(&input).unwrap().get_output();
                        let node = NodeExec::new_bootstrap(server.clone(), input, table);
                        nodes.insert(output, node);
                    }
                }
            }
        }

        let output_recv: Vec<_> = circuit
            .outputs
            .iter()
            .map(|name| nodes.get(name).unwrap().get_output())
            .collect();

        // spawn and wait for all tasks to finish
        let handles = nodes.into_values().map(|task| self.rt.spawn(task.run()));
        self.rt
            .block_on(try_join_all(handles))
            .map_err(|e| e.to_string())?;

        let mut outputs: HashMap<String, Ciphertext> = Default::default();
        for (name, mut recv) in izip!(circuit.outputs.clone(), output_recv) {
            let val = recv.blocking_recv().map_err(|e| e.to_string())?;
            outputs.insert(name, val);
        }

        Ok(outputs)
    }
}
