# [PIjN] Random String Generation Module

Cross-platform module for generating cryptographically secure random data for the “PIjN protocol” project

## Doc:
fn main() 
> Test function to demonstrate capabilities

fn generate_random_string()
> Random string generation function. In arguments specify “length” of string and char_types (about it below). Example: let random_string = generate_random_string(16, char_types)?;

CharTypes:
> Structure including generation configuration. Example of using: let char_types = CharTypes::new(digits: true, lowercase: false, ippercase: false, special: false); 
