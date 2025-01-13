use crate::lbf_circuit::{circuit::Node, Circuit};

#[derive(Default, Debug)]
pub struct CircuitStats {
    nb_input: usize,
    nb_output: usize,
    nb_lincomb: usize,
    nb_bootstrap: usize,
    nb_bootstrap_indiv: usize,
}

impl CircuitStats {
    pub fn new(circuit: &Circuit) -> CircuitStats {
        let mut stats = CircuitStats::default();

        for node in &circuit.nodes {
            match node {
                Node::Input { name: _ } => {
                    stats.nb_input += 1;
                }
                Node::LinComb {
                    inputs: _,
                    output: _,
                    coefs: _,
                    const_coef: _,
                } => {
                    stats.nb_lincomb += 1;
                }
                Node::Bootstrap {
                    input: _,
                    outputs,
                    tables: _,
                } => {
                    stats.nb_bootstrap += 1;
                    stats.nb_bootstrap_indiv += outputs.len();
                }
            }
        }

        stats.nb_output = circuit.outputs.len();

        stats
    }
}
