use client::Client;
use server::Server;
use tfhe::shortint::{gen_keys, ClassicPBSParameters};

pub mod client;
pub mod server;
pub mod test_vector;

pub fn gen_client_server(params: ClassicPBSParameters) -> (Client, Server) {
    let (client_key, server_key) = gen_keys(params);

    (Client::new(client_key), Server::new(server_key))
}

#[cfg(test)]
mod tests {
    use itertools::iproduct;
    use test_vector::TestVectorType;
    use tfhe::shortint::prelude::PARAM_MESSAGE_1_CARRY_1_KS_PBS;

    use super::*;

    #[test]
    fn lincomb() {
        let (client, server) = gen_client_server(PARAM_MESSAGE_1_CARRY_1_KS_PBS);

        for (b0, b1, b2) in iproduct!([0, 1], [0, 1], [0, 1]) {
            let ct0 = client.encrypt(b0);
            let ct1 = client.encrypt(b1);
            let ct2 = client.encrypt(b2);

            let ct = server.lincomb(&mut [ct0, ct1, ct2], &[2, 1, -3], 3);

            let val = client.decrypt(&ct);
            let exp_val = b0 as i8 * 2 + b1 as i8 * 1 + b2 as i8 * (-3) + 3;
            let exp_val = exp_val % client.pt_mod_full.0 as i8;
            let exp_val = if exp_val < 0 {
                client.pt_mod_full.0 as i8 + exp_val
            } else {
                exp_val
            } as u8;
            assert_eq!(exp_val, val);
        }
    }

    #[test]
    fn bootstrap() {
        let (client, server) = gen_client_server(PARAM_MESSAGE_1_CARRY_1_KS_PBS);

        let tv = server
            .new_test_vector(vec![true, false, true, false, false])
            .unwrap();

        for msg in 0..client.pt_mod_full.0 {
            let ct = client.encrypt(msg as u8);
            let ct_res = server.bootstrap(ct, &tv);

            let val = client.decrypt(&ct_res);
            let exp_val = if msg < server.pt_mod.0 {
                tv.test_vec_fnc(msg as u64) as u8
            } else {
                match tv.tv_type {
                    TestVectorType::Zero => 0,
                    TestVectorType::One => 1,
                    TestVectorType::Half => {
                        1 - tv.test_vec_fnc(msg as u64 % server.pt_mod.0 as u64) as u8
                    }
                }
            };
            assert_eq!(exp_val, val);
        }
    }
}
