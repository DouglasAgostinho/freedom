pub mod crypt{

    use std::fmt::Error;

    use ring::agreement::{EphemeralPrivateKey, PublicKey, UnparsedPublicKey};
    use ring::{agreement, rand};
    use ring::digest::{self, Digest};
    use aes::Aes256;
    use cbc::cipher::{block_padding::Pkcs7, BlockDecryptMut, BlockEncryptMut, KeyIvInit};
    //use hex::encode;

    type Aes256CbcEnc = cbc::Encryptor<Aes256>;
    type Aes256CbcDec = cbc::Decryptor<Aes256>;

    
    pub fn generate_own_keys() -> (EphemeralPrivateKey, PublicKey){

        // Generate Diffie-Hellman parameters and keys
        let rng = rand::SystemRandom::new();
        let private_key = agreement::EphemeralPrivateKey::generate(&agreement::X25519, &rng).unwrap();

        let public_key = private_key.compute_public_key().unwrap();

        (private_key, public_key)        
        
    }

    ///Function will get the client public key and return shared secret key
    pub fn generate_shared_key(private_key: EphemeralPrivateKey, client_public_key: UnparsedPublicKey<Vec<u8>>) -> Digest{

        // Compute shared secrets
        let shared_secret = agreement::agree_ephemeral(
            private_key,
            &agreement::UnparsedPublicKey::new(&agreement::X25519, client_public_key.as_ref()),
            //ring::error::Unspecified,
            |key_material| {
                let mut secret: Vec<u8> = vec![0u8; key_material.len()];
                secret.copy_from_slice(key_material);
                Ok::<Vec<u8>, Error>(secret).expect("Error")
            }
            ).unwrap();
    
            // Hash the shared secrets to get the final shared keys
            let shared_key = digest::digest(&digest::SHA256, &shared_secret);
            
            //println!("Shared key: {:?}", encode(shared_key.as_ref()));

            shared_key
    
    }

    pub fn encrypt(shared_key: Digest, msg: String) -> Vec<u8>{        

        // Encrypt a message
        let message = msg.as_bytes();
        let iv = [0u8; 16]; // Initialization vector (should be random in practice)

        let mut buffer = message.to_vec();
        
        // Pad buffer to the block size
        buffer.resize(message.len() + 16 - (message.len() % 16), 0);

        let cipher = Aes256CbcEnc::new_from_slices(shared_key.as_ref(), &iv).unwrap();
        let _ciphertext = cipher.encrypt_padded_mut::<Pkcs7>(&mut buffer, message.len()).unwrap();

        //println!("Encrypted message: {}", encode(ciphertext));

        buffer
    }

    pub fn decrypt(shared_key: Digest, mut buffer: Vec<u8>) -> String {

        let iv = [0u8; 16]; // Initialization vector (should be random in practice)

        // Decrypt the message
        let cipher = Aes256CbcDec::new_from_slices(shared_key.as_ref(), &iv).unwrap();
        let decrypted_ciphertext = cipher.decrypt_padded_mut::<Pkcs7>(&mut buffer).unwrap();

        let msg = String::from_utf8(decrypted_ciphertext.to_vec()).unwrap();

        //println!("Decrypted message: {:?}", msg);

        msg
    }

    pub fn test_keys() -> PublicKey{

        let rng = rand::SystemRandom::new();
        let test_private_key = agreement::EphemeralPrivateKey::generate(&agreement::X25519, &rng).unwrap();
        let test_public_key = test_private_key.compute_public_key().unwrap();

        test_public_key
    }


}