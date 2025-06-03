use rand::{rngs::OsRng, RngCore, SeedableRng, seq::index::sample, distributions::{Uniform, Distribution}};
use rand_chacha::ChaCha20Rng;
use std::fmt::Debug;



const DIGITS: &[u8] = b"0123456789";
const LOWERCASE: &[u8] = b"abcdefghijklmnopqrstuvwxyz";
const UPPERCASE: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ";
const SPEC: &[u8] = b"!@#$%^&*-_=+~><?/";



struct SecureRandom {
    rng: ChaCha20Rng,
}


impl SecureRandom {
    fn new() -> Self {
        let mut seed = [0u8; 32];
        OsRng.fill_bytes(&mut seed);
        Self {
            rng: ChaCha20Rng::from_seed(seed),
        }
    }

    fn random_index(&mut self, max: usize) -> usize {
        let dist = Uniform::from(0..max);
        dist.sample(&mut self.rng)
    }

    fn sample_indices(&mut self, len: usize, count: usize) -> Vec<usize> {
        sample(&mut self.rng, len, count).into_vec()
    }
}



struct RandomStringGenerator<'a> {
    charset: Vec<u8>,
    rng: SecureRandom,
    _marker: std::marker::PhantomData<&'a ()>,
}


impl<'a> RandomStringGenerator<'a> {
    fn new(use_digits: bool, use_lowercase: bool, use_uppercase: bool, use_spec: bool) -> Self {
        let mut charset = Vec::new();
        if use_digits {
            charset.extend_from_slice(DIGITS);
        }
        if use_lowercase {
            charset.extend_from_slice(LOWERCASE);
        }
        if use_uppercase {
            charset.extend_from_slice(UPPERCASE);
        }
        if use_spec {
            charset.extend_from_slice(SPEC);
        }

        assert!(!charset.is_empty(), "Must be selected one or more types of symbols");

        Self {
            charset,
            rng: SecureRandom::new(),
            _marker: std::marker::PhantomData,
        }
    }


    fn generate(&mut self, length: usize) -> String {
        (0..length)
            .map(|_| {
                let idx = self.rng.random_index(self.charset.len());
                self.charset[idx] as char
            })
            .collect()
    }
}



struct RandomSelector<T> {
    rng: SecureRandom,
    _marker: std::marker::PhantomData<T>,
}


impl<T> RandomSelector<T>
where
    T: Debug + Clone,
{
    fn new() -> Self {
        Self {
            rng: SecureRandom::new(),
            _marker: std::marker::PhantomData,
        }
    }

    fn choose(&mut self, data: &[T], count: usize) -> Vec<T> {
        assert!(
            count <= data.len(),
            "Cant select more elements than in source"
        );

        let indices = self.rng.sample_indices(data.len(), count);
        indices.into_iter().map(|i| data[i].clone()).collect()
    }
}



// Main functions ==============================
pub fn generate_random_string(use_digits: bool, use_lowercase: bool, use_uppercase: bool, use_spec: bool, length: usize) -> String {
    RandomStringGenerator::new(use_digits, use_lowercase, use_uppercase, use_spec).generate(length)
}


pub fn generate_random_choose<T>(items: Vec<T>, count_of_items: usize) -> Vec<T>
where
    T: Clone + std::fmt::Debug,
{
    let mut selector = RandomSelector::new();
    selector.choose(&items, count_of_items)
}



// test (DO NOT USE ON PROD)
fn main() {
    let random_str = generate_random_string(true, true, true, true, 16);
    println!("[TEST] generate random string: {}", random_str);

    let items = vec![1, 2, 3];
    let random_select = generate_random_choose(items, 2);
    println!("[TEST] generate random choose: {:?}", random_select);
}
