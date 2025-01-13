use std::collections::HashMap;

use crate::{
    circuit::Circuit,
    tfhe::{Ciphertext, Server},
};

pub trait FbsExec {
    fn eval(
        &self,
        server: Server,
        circuit: &Circuit,
        inputs: HashMap<String, Ciphertext>,
    ) -> Result<HashMap<String, Ciphertext>, String>;
}
