use clap::Parser;
use std::io::{ Error, ErrorKind };

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Args {
    #[arg(short, long)]
    pub path: String,

    #[arg(short, long)]
    pub algorythm: String
}

#[derive(Debug)]
pub enum Algo {
    Md5,
    Sha256,
    Blake3
}

pub fn parse_algo(algo: &str) -> Result<Algo, Box<dyn std::error::Error>> {
    match algo.to_lowercase().as_str() {
        "md5" => Ok(Algo::Md5),
        "sha256" => Ok(Algo::Sha256),
        "blake3" => Ok(Algo::Blake3),
        _ => Err(
            Box::new(
                Error::new(
                    ErrorKind::InvalidInput, 
                    "Invalid Algorythm. Use: md5, sha256 or blake3"
                )
            )
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_algo_md5_lowercase() {
        // Test 1: Lowercase input (base case)
        let result = parse_algo("md5");
        assert!(result.is_ok(), "Expected Ok, received Err");
        assert!(matches!(result.unwrap(), Algo::Md5));
    }

    #[test]
    fn test_parse_algo_sha256_uppercase() {
        // Test 2: Input in uppercase letters (checks case-insensitivity)
        let result = parse_algo("SHA256");
        assert!(result.is_ok(), "Expected Ok, received Err");
        assert!(matches!(result.unwrap(), Algo::Sha256));
    }

    #[test]
    fn test_parse_algo_blake3_mixed_case() {
        // Test 3: Mixed input (additional edge case)
        let result = parse_algo("BlaKe3");
        assert!(result.is_ok(), "Expected Ok, received Err");
        assert!(matches!(result.unwrap(), Algo::Blake3));
    }

    #[test]
    fn test_parse_algo_invalid() {
        // Test 4: Invalid input (checks error handling)
        let result = parse_algo("invalid");
        assert!(result.is_err(), "Expected Err, received Ok");
        
        // Optional: Verify that the error message contains what is expected
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("Invalid Algorythm"));
    }
}
