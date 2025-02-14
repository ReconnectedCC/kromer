use rand::{distr::Uniform, Rng};
use sha2::{Digest, Sha256};

pub fn generate_random_password() -> String {
    // Define the character set, including letters, digits, underscores, and hyphens
    let charset: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ\
                           abcdefghijklmnopqrstuvwxyz\
                           0123456789_-";
    let mut rng = rand::rng();
    let dist = Uniform::try_from(0..charset.len()).expect("failed to sample values between bounds");

    // Generate a random string of 32 characters
    (0..32).map(|_| charset[rng.sample(dist)] as char).collect()
}

pub fn sha256(data: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data.as_bytes());
    hex::encode(hasher.finalize())
}

pub fn double_sha256(data: &str) -> String {
    let first_hash = sha256(data);
    sha256(&first_hash)
}

fn hex_to_base36(byte: u8) -> char {
    let res_byte = match byte / 7 {
        byte @ 0..=9 => byte + b'0',
        byte @ 10..=35 => byte + b'a' - 10,
        36 => b'e',
        _ => unreachable!(),
    };
    res_byte as char
}

pub fn make_v2_address(key: &str, address_prefix: &str) -> String {
    let mut protein = [0u8; 9];
    let mut used = [false; 9];
    let mut chain = String::from(address_prefix);
    let mut hash = double_sha256(key);

    for i in 0..9 {
        protein[i] = u8::from_str_radix(&hash[0..2], 16).unwrap();
        hash = double_sha256(&hash);
    }

    let mut i = 0;
    while i < 9 {
        let start = 2 * i;
        let end = start + 2;
        let hex_part = &hash[start..end];
        let num = u8::from_str_radix(hex_part, 16).unwrap();
        let index = (num % 9) as usize;

        if used[index] {
            hash = sha256(&hash);
        } else {
            chain.push(hex_to_base36(protein[index]));
            used[index] = true;
            i += 1;
        }
    }

    chain
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_known_values() {
        assert_eq!(make_v2_address("test123", "k"), "krcgbmalxg");
        assert_eq!(make_v2_address("0", "k"), "kzbdy8rmok");
        assert_eq!(make_v2_address("1", "k"), "k4om3ewezk");
        assert_eq!(make_v2_address("2", "k"), "kd18lv0b6u");
        assert_eq!(make_v2_address("3", "k"), "krdfu99fep");
        assert_eq!(make_v2_address("4", "k"), "k8kl0fyol5");
        assert_eq!(make_v2_address("5", "k"), "kl996ygs97");
        assert_eq!(make_v2_address("6", "k"), "k926k4tgmh");
        assert_eq!(make_v2_address("7", "k"), "k6o8rgjqi2");
        assert_eq!(make_v2_address("8", "k"), "knvvk3kahp");
        assert_eq!(make_v2_address("9", "k"), "kv2k3ja3o9");
    }
}
