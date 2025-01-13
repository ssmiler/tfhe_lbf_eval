use std::{collections::HashMap, sync::Arc};

use crate::{
    circuit::{circuit::Node, Circuit},
    tfhe::{Ciphertext, Server},
};

use dagrs::{log, Action, Dag, DefaultTask, EnvVar, Input, LogLevel, Output, RunningError, Task};

use super::fbs_exec::FbsExec;

#[derive(Default)]
pub struct FbsExecPar;

struct InputAction {
    name: String,
}

impl Action for InputAction {
    fn run(&self, _input: Input, env: Arc<EnvVar>) -> Result<Output, RunningError> {
        let val = env
            .get::<Ciphertext>(&self.name)
            .ok_or(RunningError::new("Cannot find input".into()))?;
        Ok(Output::new(val))
    }
}

struct LinCombAction {
    coefs: Vec<i8>,
    const_coef: i8,
}

impl Action for LinCombAction {
    fn run(&self, input: Input, env: Arc<EnvVar>) -> Result<Output, RunningError> {
        let server = env.get::<Arc<Server>>("TFHE_SERVER").unwrap();
        let inp_vals = input.get_iter().map(|e| e.get::<Ciphertext>().unwrap());
        let val = server.lincomb(inp_vals, &self.coefs, self.const_coef);
        Ok(Output::new(val))
    }
}

struct BootstrapAction {
    table: Vec<bool>,
}

impl Action for BootstrapAction {
    fn run(&self, input: Input, env: Arc<EnvVar>) -> Result<Output, RunningError> {
        let server = env.get::<Arc<Server>>("TFHE_SERVER").unwrap();
        let inp = input
            .get_iter()
            .next()
            .unwrap()
            .get::<Ciphertext>()
            .unwrap();
        let val = server.bootstrap(
            inp.clone(),
            &server.new_test_vector(self.table.clone()).unwrap(),
        );
        Ok(Output::new(val))
    }
}

struct OutputAction {
    outputs: Vec<String>,
}

impl Action for OutputAction {
    fn run(&self, input: Input, env: Arc<EnvVar>) -> Result<Output, RunningError> {
        let outputs: HashMap<String, Ciphertext> = self
            .outputs
            .iter()
            .zip(input.get_iter())
            .map(|(name, val)| (name.clone(), val.get::<Ciphertext>().unwrap().clone()))
            .collect();

        Ok(Output::new(outputs))
    }
}

impl FbsExecPar {
    pub fn new() -> Self {
        Self
    }

    pub fn new_boxed() -> Box<Self> {
        Box::new(Self)
    }

    fn build_dag(&self, circuit: &Circuit) -> Dag {
        let mut name2id = HashMap::<&str, usize>::new();
        let mut tasks = Vec::<DefaultTask>::new();

        for node in &circuit.nodes {
            match node {
                Node::Input { name } => {
                    let task = DefaultTask::new(
                        InputAction {
                            name: name.to_owned(),
                        },
                        name,
                    );
                    name2id.insert(name, task.id());

                    tasks.push(task);
                }
                Node::LinComb {
                    coefs,
                    const_coef,
                    output,
                    inputs,
                } => {
                    let mut task = DefaultTask::new(
                        LinCombAction {
                            coefs: coefs.clone(),
                            const_coef: *const_coef,
                        },
                        output,
                    );
                    name2id.insert(output, task.id());

                    // add predecessor relations
                    let predecessors: Vec<_> = inputs
                        .iter()
                        .map(|e| *name2id.get(e.as_str()).unwrap())
                        .collect();
                    task.set_predecessors_by_id(predecessors.as_slice());

                    tasks.push(task);
                }
                Node::Bootstrap {
                    input,
                    outputs,
                    tables,
                } => {
                    for (output, table) in outputs.iter().zip(tables) {
                        let mut task = DefaultTask::new(
                            BootstrapAction {
                                table: table.clone(),
                            },
                            output,
                        );
                        name2id.insert(output, task.id());

                        // add predecessor relations
                        task.set_predecessors_by_id(&[*name2id.get(input.as_str()).unwrap()]);

                        tasks.push(task);
                    }
                }
            }
        }

        let mut output_task = DefaultTask::new(
            OutputAction {
                outputs: circuit.outputs.clone(),
            },
            "BUILD_OUTPUT_MAP",
        );

        // add predecessor relations
        let predecessors: Vec<_> = circuit
            .outputs
            .iter()
            .map(|e| *name2id.get(e.as_str()).unwrap())
            .collect();
        output_task.set_predecessors_by_id(predecessors.as_slice());

        tasks.push(output_task);

        Dag::with_tasks(tasks)
    }
}

impl FbsExec for FbsExecPar {
    fn eval(
        &self,
        server: Server,
        circuit: &Circuit,
        inputs: HashMap<String, Ciphertext>,
    ) -> Result<HashMap<String, Ciphertext>, String> {
        log::init_logger(LogLevel::Info, None);

        let mut dag = self.build_dag(circuit);

        // Set a global environment variable for this dag.
        let mut env = EnvVar::new();
        env.set("TFHE_SERVER", Arc::new(server));
        inputs
            .into_iter()
            .for_each(|(name, val)| env.set(&name, val));
        dag.set_env(env);

        dag.start().map_err(|e| e.to_string())?;

        dag.get_result::<HashMap<String, Ciphertext>>()
            .ok_or("Error".to_owned())
    }
}
