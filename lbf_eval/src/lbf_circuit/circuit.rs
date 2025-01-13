use std::collections::HashSet;

#[derive(Debug, Clone)]
pub enum Node {
    Input {
        name: String,
    },
    LinComb {
        inputs: Vec<String>,
        output: String,
        coefs: Vec<i8>,
        const_coef: i8,
    },
    Bootstrap {
        input: String,
        outputs: Vec<String>,
        tables: Vec<Vec<bool>>,
    },
}

#[derive(Debug, Default)]
pub struct Circuit {
    pub inputs: Vec<String>,
    pub nodes: Vec<Node>, // first nodes are inputs
    pub outputs: Vec<String>,
}

impl Circuit {
    pub fn new() -> Circuit {
        Circuit::default()
    }

    pub fn add_input(&mut self, name: String) {
        self.inputs.push(name.clone());
        self.nodes.push(Node::Input { name });
    }

    pub fn add_output(&mut self, name: String) {
        self.outputs.push(name);
    }

    pub fn add_lincomb(
        &mut self,
        output: String,
        inputs: Vec<String>,
        coefs: Vec<i8>,
        const_coef: i8,
    ) {
        self.nodes.push(Node::LinComb {
            inputs,
            output,
            coefs,
            const_coef,
        });
    }

    pub fn add_bootstrap(&mut self, outputs: Vec<String>, input: String, tables: Vec<Vec<bool>>) {
        self.nodes.push(Node::Bootstrap {
            input,
            outputs,
            tables,
        });
    }

    pub fn check(self) -> Result<Circuit, String> {
        // traverse circuit and check if name exist or not defined twice
        //  and other consistency checks
        let mut visited_names = HashSet::<String>::new();
        let mut used_nodes = HashSet::<String>::new();

        fn add_new_name(visited_names: &mut HashSet<String>, name: &String) -> Result<(), String> {
            if visited_names.contains(name) {
                return Err(format!("Name {} already defined", name));
            }
            visited_names.insert(name.clone());
            Ok(())
        }

        fn check_name_exists(visited_names: &HashSet<String>, name: &String) -> Result<(), String> {
            if !visited_names.contains(name) {
                return Err(format!("Name {} does not exist", name));
            }
            Ok(())
        }

        for node in &self.nodes {
            match node {
                Node::Input { name } => {
                    add_new_name(&mut visited_names, name)?;
                }
                Node::LinComb { inputs, output, .. } => {
                    for input in inputs {
                        check_name_exists(&visited_names, input)?;
                        used_nodes.insert(input.clone());
                    }
                    add_new_name(&mut visited_names, output)?;
                }
                Node::Bootstrap { input, outputs, .. } => {
                    check_name_exists(&visited_names, input)?;
                    used_nodes.insert(input.clone());
                    for output in outputs {
                        add_new_name(&mut visited_names, output)?;
                    }
                }
            }
        }

        // check output once all nodes are visited
        for name in &self.outputs {
            check_name_exists(&visited_names, name)?;
            used_nodes.insert(name.clone());
        }

        if visited_names.len() > used_nodes.len() {
            println!("AAA {:?}", visited_names.difference(&used_nodes));
            return Err("Circuit has dangling nodes".to_owned());
        }

        Ok(self)
    }
}
