/*
Random string generation module for PIjN proticol project
Developer: Urban Egor
Version: 2.3.20 a
*/

use std::io;
use rand::rngs::OsRng;
use rand_core::TryRngCore;



const CHACHA_BLOCK_SIZE: usize = 64; 
const CHACHA_KEY_SIZE: usize = 32; 
const CHACHA_NONCE_SIZE: usize = 12; 
const CHACHA_ROUNDS: usize = 12; 



#[derive(Clone, Copy)]
pub struct CharTypes {
    pub digits: bool,      // 0-9
    pub lowercase: bool,   // a-z
    pub uppercase: bool,   // A-Z
    pub special: bool,     // !@#$%^&*()_+-=[]{}|;:,.<>?
}



impl CharTypes {
    pub fn new(digits: bool, lowercase: bool, uppercase: bool, special: bool) -> Self {
        CharTypes {
            digits,
            lowercase,
            uppercase,
            special,
        }
    }
}



struct ChaChaRng {
    state: [u32; 16],
    output: [u8; CHACHA_BLOCK_SIZE], 
    output_pos: usize,
}


impl ChaChaRng {
    fn new() -> io::Result<Self> {
        let mut key = [0u8; CHACHA_KEY_SIZE];
        let mut nonce = [0u8; CHACHA_NONCE_SIZE];
    
        let mut os_rng = OsRng;
        os_rng.try_fill_bytes(&mut key).map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
        os_rng.try_fill_bytes(&mut nonce).map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
        
    
        let mut state = [0u32; 16];
        state[0] = 0x61707865;
        state[1] = 0x3320646e; 
        state[2] = 0x79622d32;
        state[3] = 0x6b206574;
    
        for i in 0..8 {
            state[4 + i] = u32::from_le_bytes([
                key[i * 4],
                key[i * 4 + 1],
                key[i * 4 + 2],
                key[i * 4 + 3],
            ]);
        }
        state[12] = 0;
    
        for i in 0..3 {
            state[13 + i] = u32::from_le_bytes([
                nonce[i * 4],
                nonce[i * 4 + 1],
                nonce[i * 4 + 2],
                nonce[i * 4 + 3],
            ]);
        }
    
        Ok(ChaChaRng {
            state,
            output: [0u8; CHACHA_BLOCK_SIZE],
            output_pos: CHACHA_BLOCK_SIZE,
        })
    }
    

    fn quarter_round(state: &mut [u32; 16], a: usize, b: usize, c: usize, d: usize) {
        state[a] = state[a].wrapping_add(state[b]); state[d] ^= state[a]; state[d] = state[d].rotate_left(16);
        state[c] = state[c].wrapping_add(state[d]); state[b] ^= state[c]; state[b] = state[b].rotate_left(12);
        state[a] = state[a].wrapping_add(state[b]); state[d] ^= state[a]; state[d] = state[d].rotate_left(8);
        state[c] = state[c].wrapping_add(state[d]); state[b] ^= state[c]; state[b] = state[b].rotate_left(7);
    }

    fn generate_block(&mut self) {
        let mut working_state = self.state;

        for _ in 0..CHACHA_ROUNDS / 2 {
            Self::quarter_round(&mut working_state, 0, 4, 8, 12);
            Self::quarter_round(&mut working_state, 1, 5, 9, 13);
            Self::quarter_round(&mut working_state, 2, 6, 10, 14);
            Self::quarter_round(&mut working_state, 3, 7, 11, 15);

            Self::quarter_round(&mut working_state, 0, 5, 10, 15);
            Self::quarter_round(&mut working_state, 1, 6, 11, 12);
            Self::quarter_round(&mut working_state, 2, 7, 8, 13);
            Self::quarter_round(&mut working_state, 3, 4, 9, 14);
        }

        for i in 0..16 {
            working_state[i] = working_state[i].wrapping_add(self.state[i]);
        }

        for (i, word) in working_state.iter().enumerate() {
            self.output[i * 4..(i + 1) * 4].copy_from_slice(&word.to_le_bytes());
        }

        self.state[12] = self.state[12].wrapping_add(1);
        self.output_pos = 0;
    }

    fn fill_bytes(&mut self, dest: &mut [u8]) {
        let mut remaining = dest.len();
        let mut dest_pos = 0;

        while remaining > 0 {
            if self.output_pos >= CHACHA_BLOCK_SIZE {
                self.generate_block();
            }

            let to_copy = (CHACHA_BLOCK_SIZE - self.output_pos).min(remaining);
            dest[dest_pos..dest_pos + to_copy]
                .copy_from_slice(&self.output[self.output_pos..self.output_pos + to_copy]);
            self.output_pos += to_copy;
            dest_pos += to_copy;
            remaining -= to_copy;
        }
    }

    fn gen_range(&mut self, max: u64) -> u64 {
        let mut bytes = [0u8; 8];
        self.fill_bytes(&mut bytes);
        let mut val = u64::from_le_bytes(bytes);
        let range = u64::MAX / max * max;
        while val >= range {
            self.fill_bytes(&mut bytes);
            val = u64::from_le_bytes(bytes);
        }
        val % max
    }
}



pub fn generate_random_string(length: usize, char_types: CharTypes) -> io::Result<String> {
    let mut rng = ChaChaRng::new()?;
    let mut charset = Vec::new();

    if char_types.digits {
        charset.extend(b"0123456789".iter().copied());
    }
    if char_types.lowercase {
        charset.extend(b"abcdefghijklmnopqrstuvwxyz".iter().copied());
    }
    if char_types.uppercase {
        charset.extend(b"ABCDEFGHIJKLMNOPQRSTUVWXYZ".iter().copied());
    }
    if char_types.special {
        charset.extend(b"!@#$%^&*()_+-=[]{}|;:,.<>?".iter().copied());
    }

    if charset.is_empty() {
        return Err(io::Error::new(io::ErrorKind::InvalidInput, "At least one character type must be selected"));
    }

    let mut result = String::with_capacity(length);
    for _ in 0..length {
        let idx = rng.gen_range(charset.len() as u64) as usize;
        result.push(charset[idx] as char);
    }

    Ok(result)
}



fn main() -> io::Result<()> {
    let char_types = CharTypes::new(true, false, false, false); 
    let random_string = generate_random_string(16, char_types)?;
    println!("{}", random_string);

    Ok(())
}