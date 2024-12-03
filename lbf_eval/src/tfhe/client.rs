use tfhe::shortint::{Ciphertext, ClientKey, MessageModulus};

pub struct Client {
    client_key: ClientKey,
    pt_mod_full: MessageModulus,
}

impl Client {
    pub fn new(client_key: ClientKey) -> Client {
        let pt_mod_full = MessageModulus(
            2 * client_key.parameters.message_modulus().0
                * client_key.parameters.carry_modulus().0,
        );
        Client {
            client_key,
            pt_mod_full,
        }
    }

    pub fn encrypt(&self, msg: u8) -> Ciphertext {
        // 2x msg modulus to go into the negacyclic part (padding bit)
        self.client_key
            .encrypt_with_message_modulus(msg as u64, self.pt_mod_full)
    }

    pub fn decrypt(&self, ct: &Ciphertext) -> u8 {
        let msg = self.client_key.decrypt_message_and_carry(&ct);
        assert!((msg as usize) < self.pt_mod_full.0);
        msg as u8
    }
}
