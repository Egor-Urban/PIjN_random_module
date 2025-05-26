# [PIjN] Random data generation and selection module

**Developer:** Urban Egor  
**Version:** 4.5.38 a

This module provides secure random string generation and random selection functionality for the PIjN protocol.

---

## Features

- Cryptographically secure random string generation using a customizable character set.
- Random selection of items from a list without repetition.
- Based on `ChaCha20Rng` seeded with system entropy (`OsRng`).

---

## Modules and Structures

### `SecureRandom`

Internal wrapper over `ChaCha20Rng`:

- `new()`: Initializes with a 32-byte secure random seed.
- `random_index(max: usize) -> usize`: Returns a random index in `[0, max)`.
- `sample_indices(len: usize, count: usize) -> Vec<usize>`: Returns unique sample indices.

---

### `RandomStringGenerator`

Used to generate a secure random string from a customizable charset.

#### Constructor:
```rust
RandomStringGenerator::new(
    use_digits: bool,
    use_lowercase: bool,
    use_uppercase: bool,
    use_spec: bool
)
```

### Method:

    generate(length: usize) -> String  
    Generates a string of given length.

### Character groups:

    DIGITS:    0–9  
    LOWERCASE: a–z  
    UPPERCASE: A–Z  
    SPEC:      !@#$%^&*-_=+~><?/

---

### RandomSelector<T>

Used to randomly choose a subset of elements from a given list (without replacement).

#### Constructor:

    RandomSelector::new()

#### Method:

    choose(data: &[T], count: usize) -> Vec<T>  
    T must implement Clone and Debug.

---

## Public Functions

### generate_random_string(...) -> String

Wrapper to create and return a random string.  
Arguments: same as `RandomStringGenerator::new`.

### generate_random_choose<T>(items: Vec<T>, count_of_items: usize) -> Vec<T>

Wrapper to randomly select `count_of_items` elements from `items`.

---

## Test Usage (Do not use in production)

```rust
fn main() {
    let random_str = generate_random_string(true, true, true, true, 16);
    println!("[TEST] generate random string: {}", random_str);

    let items = vec![1, 2, 3];
    let random_select = generate_random_choose(items, 2);
    println!("[TEST] generate random choose: {:?}", random_select);
}
