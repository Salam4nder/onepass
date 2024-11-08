use rand::Rng;

const UPPERCASE:     &[u8; 26] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ";
const LOWERCASE:     &[u8; 26] = b"abcdefghijklmnopqrstuvwxyz";
const NUMBERS:       &[u8; 10] = b"0123456789";
const SPECIAL_CHARS: &[u8; 26] = b"!@#$%^&*()-_=+[]{}|;:,.<>?";

fn suggest(length: usize) -> String {
    let mut rng = rand::thread_rng();

    // Make sure we include at least one of each character type
    let mut password = vec![
        UPPERCASE[rng.gen_range(0..UPPERCASE.len())] as char,
        LOWERCASE[rng.gen_range(0..LOWERCASE.len())] as char,
        NUMBERS[rng.gen_range(0..NUMBERS.len())] as char,
        SPECIAL_CHARS[rng.gen_range(0..SPECIAL_CHARS.len())] as char,
    ];

    let all_chars: Vec<u8> = UPPERCASE.iter()
        .chain(LOWERCASE)
        .chain(NUMBERS)
        .chain(SPECIAL_CHARS)
        .copied()
        .collect();

    // Fill the rest of the password
    for _ in 4..length {
        let random_char = all_chars[rng.gen_range(0..all_chars.len())] as char;
        password.push(random_char);
    }

    // Shuffle the password vector by swapping random elements
    for i in 0..password.len() {
        let j = rng.gen_range(0..password.len());
        password.swap(i, j);
    }

    // Collect characters into a final password string
    password.into_iter().collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn suggest_test() {
        for _ in 0..15 {
            let mut found_number      : bool = false;
            let mut found_uppercase   : bool = false;
            let mut found_lowercase   : bool = false;
            let mut found_special_char: bool = false;
            let example = suggest(14);
            for v in example.as_bytes() {
                if UPPERCASE.contains(v) {
                    found_uppercase = true;
                }
                if LOWERCASE.contains(v) {
                    found_lowercase = true;
                }
                if NUMBERS.contains(v) {
                    found_number = true;
                }
                if SPECIAL_CHARS.contains(v) {
                    found_special_char = true;
                }
            }
            if example.len() != 14 {
                   panic!("password not 14 chars")
            }

            if !found_number       ||
               !found_special_char ||
               !found_lowercase    ||
               !found_uppercase    {
                   panic!("password not strong")
               }
        };
    }
}
