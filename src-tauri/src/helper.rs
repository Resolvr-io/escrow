use std::str::FromStr;

pub fn bitcoin_xonly_to_nostr_xonly(
    key: &bitcoin::XOnlyPublicKey,
) -> nostr_sdk::prelude::XOnlyPublicKey {
    nostr_sdk::prelude::XOnlyPublicKey::from_str(&key.to_string()).unwrap()
}

pub fn nostr_xonly_to_bitcoin_xonly(
    key: &nostr_sdk::prelude::XOnlyPublicKey,
) -> bitcoin::XOnlyPublicKey {
    bitcoin::XOnlyPublicKey::from_str(&key.to_string()).unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;
    use secp256k1_zkp::rand::thread_rng;
    use secp256k1_zkp::Secp256k1;

    #[test]
    fn test_bitcoin_xonly_round_trip() {
        let secret_key = bitcoin::secp256k1::SecretKey::new(&mut thread_rng());
        let secp = Secp256k1::new();
        let bitcoin_key = secret_key.x_only_public_key(&secp).0;
        let nostr_key = bitcoin_xonly_to_nostr_xonly(&bitcoin_key);
        let back_to_bitcoin_key = nostr_xonly_to_bitcoin_xonly(&nostr_key);
        assert_eq!(bitcoin_key, back_to_bitcoin_key);
    }

    #[test]
    fn test_nostr_xonly_round_trip() {
        let secret_key = nostr_sdk::prelude::SecretKey::new(&mut thread_rng());
        let secp = nostr_sdk::secp256k1::Secp256k1::new();
        let nostr_key = secret_key.x_only_public_key(&secp).0;
        let bitcoin_key = nostr_xonly_to_bitcoin_xonly(&nostr_key);
        let back_to_nostr_key = bitcoin_xonly_to_nostr_xonly(&bitcoin_key);
        assert_eq!(nostr_key, back_to_nostr_key);
    }
}
