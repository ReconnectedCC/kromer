use rand::{distributions::Uniform, thread_rng, Rng};
use sha2::{Digest, Sha256};

pub fn generate_random_password() -> String {
    // Define the character set, including letters, digits, underscores, and hyphens
    let charset: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ\
                           abcdefghijklmnopqrstuvwxyz\
                           0123456789_-";
    let mut rng = thread_rng();
    let dist = Uniform::from(0..charset.len());

    // Generate a random string of 32 characters
    (0..32).map(|_| charset[rng.sample(dist)] as char).collect()
}

pub fn sha256(input: &str) -> String {
    let mut hasher = Sha256::new();

    hasher.update(input.as_bytes());

    format!("{:x}", hasher.finalize())
}

pub fn double_sha256(input: &str) -> String {
    sha256(&sha256(input))
}

pub fn hex_to_base36(input: u8) -> char {
    let byte = 48 + (input / 7);

    let adjusted_byte = if byte + 39 > 112 {
        101 // 'e'
    } else if byte > 57 {
        byte + 39
    } else {
        byte
    };

    char::from(adjusted_byte)
}

pub fn make_v2_address(key: &str, address_prefix: &str) -> String {
    let mut chars = vec![String::new(); 9];
    let mut chain = address_prefix.to_string();
    let mut hash = double_sha256(key);

    for item in chars.iter_mut().take(8) {
        *item = hash[..2].to_string();
        hash = double_sha256(&hash);
    }

    let mut i = 0;
    while i < 8 {
        let index = usize::from_str_radix(&hash[(2 * i)..(2 + 2 * i)], 16);
        if index.is_ok() {
            let index = index.unwrap() % 9;
            if chars[index].is_empty() {
                hash = sha256(&hash);
            } else {
                let char_value = u8::from_str_radix(&chars[index], 16).unwrap();
                chain.push(hex_to_base36(char_value));
                chars[index].clear();
                i += 1;
            }
        }
    }

    chain
}
