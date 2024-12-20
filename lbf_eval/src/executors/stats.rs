use std::{collections::HashMap, os::linux::raw::stat};

use itertools::izip;

use crate::circuit::{circuit::Node, Circuit};

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
                Node::Input { name } => {
                    stats.nb_input += 1;
                }
                Node::LinComb {
                    inputs,
                    output,
                    coefs,
                    const_coef,
                } => {
                    stats.nb_lincomb += 1;
                }
                Node::Bootstrap {
                    input,
                    outputs,
                    tables,
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
