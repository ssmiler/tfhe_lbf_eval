use super::test_vector::{TestVector, TestVectorType};
use itertools::izip;
use tfhe::{
    core_crypto::prelude::{lwe_ciphertext_plaintext_add_assign, GlweCiphertext, Plaintext},
    shortint::{
        parameters::Degree, server_key::LookupTableOwned, Ciphertext, MessageModulus, ServerKey,
    },
};

pub struct Server {
    server_key: ServerKey,
    pub pt_mod: MessageModulus,
    pt_mod_full: MessageModulus,
}

impl Server {
    pub fn new(server_key: ServerKey) -> Server {
        let pt_mod = MessageModulus(server_key.message_modulus.0 * server_key.carry_modulus.0);
        let pt_mod_full = MessageModulus(2 * pt_mod.0);
        Server {
            server_key,
            pt_mod,
            pt_mod_full,
        }
    }

    pub fn new_test_vector(&self, val: Vec<bool>) -> Result<TestVector, String> {
        TestVector::new(val, self.pt_mod.0)
    }

    fn unchecked_scalar_add_halves_assign(&self, ct: &mut Ciphertext, scalar: u8) {
        let delta = (1_u64 << 63) / self.pt_mod_full.0 as u64;
        let shift_plaintext = u64::from(scalar) * delta;
        let encoded_scalar = Plaintext(shift_plaintext);
        lwe_ciphertext_plaintext_add_assign(&mut ct.ct, encoded_scalar);

        ct.degree = Degree::new(ct.degree.get() + scalar as usize);
    }

    fn build_acc(&self, tv: &TestVector) -> LookupTableOwned {
        let mut acc = GlweCiphertext::new(
            0,
            self.server_key.bootstrapping_key.glwe_size(),
            self.server_key.bootstrapping_key.polynomial_size(),
            self.server_key.ciphertext_modulus,
        );

        let mut acc_view = acc.as_mut_view();

        acc_view.get_mut_mask().as_mut().fill(0);

        // Modulus of the msg contained in the msg bits and operations buffer
        let modulus_sup = self.server_key.message_modulus.0 * self.server_key.carry_modulus.0;

        // N/(p/2) = size of each block
        let box_size = self.server_key.bootstrapping_key.polynomial_size().0 / modulus_sup;

        // Value of the shift we multiply our messages by
        let delta = (1_u64 << 63) / modulus_sup as u64;

        let mut body = acc_view.get_mut_body();
        let accumulator_u64 = body.as_mut();

        // Tracking the max value of the function to define the degree later
        let mut max_value = 0;

        let f_delta = match tv.tv_type {
            TestVectorType::Zero => 0,
            TestVectorType::Half => delta >> 1,
            TestVectorType::One => delta,
        };

        for i in 0..modulus_sup {
            let index = i * box_size;
            let f_eval = tv.test_vec_fnc(i as u64);
            max_value = max_value.max(f_eval);
            accumulator_u64[index..index + box_size].fill((f_eval * delta).wrapping_sub(f_delta));
        }

        LookupTableOwned {
            acc,
            degree: Degree::new(max_value as usize),
        }
    }

    pub fn eval_bootstrap(&self, mut val: Ciphertext, tv: &TestVector) -> Ciphertext {
        let acc = self.build_acc(tv);

        // add delta/2 to compensate for non-rotated accumulator
        self.unchecked_scalar_add_halves_assign(&mut val, 1);

        // evaluate function
        let mut res = self.server_key.apply_lookup_table(&val, &acc);

        // shift evaluated function back by delta or delta/2
        match tv.tv_type {
            TestVectorType::Half => {
                self.unchecked_scalar_add_halves_assign(&mut res, 1);
            }
            TestVectorType::One => {
                self.unchecked_scalar_add_halves_assign(&mut res, 2);
            }
            _ => (),
        }

        res
    }

    pub fn lincomb(&self, cts: &[&Ciphertext], coefs: &[i8], const_coef: i8) -> Ciphertext {
        assert!(cts.len() == coefs.len());
        let res = self.server_key.unchecked_create_trivial(const_coef as u64);
        let res = izip!(cts.into_iter(), coefs).fold(
            res,
            |mut acc, (ct, coef)| {
                if *coef > 0 {
                    let ct = self.server_key.unchecked_scalar_mul(ct, *coef as u8);
                    self.server_key.unchecked_add_assign(&mut acc, &ct)
                } else if *coef < 0 {
                    let ct = self.server_key.unchecked_scalar_mul(ct, (-*coef) as u8);
                    self.server_key.unchecked_sub_assign(&mut acc, &ct)
                } else {
                    eprintln!("Unexpected zero coefficient");
                }
                acc
            },
        );
        res
    }
}
