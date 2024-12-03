use itertools::iproduct;
use lbf_eval::tfhe::gen_client_server;
use tfhe::shortint::prelude::*;

fn main() {
    let (client, server) = gen_client_server(PARAM_MESSAGE_1_CARRY_1_KS_PBS);

    let tv = server
        .new_test_vector(vec![true, false, true, false, true])
        .unwrap();
    println!("TV {:?}", tv);

    for (b0, b1, b2) in iproduct!([0, 1], [0, 1], [0, 1]) {
        let ct0 = client.encrypt(b0);
        let ct1 = client.encrypt(b1);
        let ct2 = client.encrypt(b2);

        let ct = server.lincomb(&mut [ct0, ct1, ct2], &[2, 1, 1], 0);
        let ct_msg = client.decrypt(&ct);

        let ct_res = server.bootstrap(ct, &tv);

        let output = client.decrypt(&ct_res);

        println!(
            "OK {} {} {} {} {} {}",
            b0, b1, b2, ct_msg, output, ct_res.message_modulus.0
        );
    }

    // for msg in 0..16 {
    //     let ct = client.encrypt(msg);
    //     let ct_msg = client.decrypt(&ct);

    //     let ct_res = server.eval_bootstrap(ct, &tv);

    //     let output = client.decrypt(&ct_res);

    //     println!(
    //         "OK {} {} {} {}",
    //         msg, ct_msg, output, ct_res.message_modulus.0
    //     );
    //     // break;
    // }
}
