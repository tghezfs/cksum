use std::fs::File;
use std::path::Path;
use std::io::{ BufReader, Read, Write };

use tempfile::NamedTempFile;
use sha2::{ Sha256, Digest };

use super::cli::Algo;

pub fn process_file(
    base_path: &Path,
    file_path: &Path,
    mut buffer: &mut [u8],
    algo: &Algo,
    temp_file: &mut NamedTempFile 
) -> Result<(), Box<dyn std::error::Error>> {
    let file = File::open(file_path)?;
    let mut reader = BufReader::new(file);

    let rel_path = file_path
        .strip_prefix(base_path)
        .expect("Base dir should be a prefix of the file path")
        .to_string_lossy()
        .to_string();

    let line = match algo {
        Algo::Md5 => {
            let mut ctx = md5::Context::new();

            loop {
                let read = reader.read(&mut buffer)?;
                if read == 0 { break; }
                ctx.consume(&buffer[..read])
            }

            let digest = ctx.finalize();
            let hash = format!("{:x}", digest);

            format!("md5 {} {}", hash, rel_path)
        },
        Algo::Sha256 => {
            let mut hasher = Sha256::new();

            loop {
                let read = reader.read(&mut buffer)?;
                if read == 0 { break; }
                hasher.update(&buffer[..read]);
            }

            let hash: String = hasher
                .finalize()
                .iter()
                .map(|b| format!("{:02x}", b))
                .collect();

            format!("sha256 {} {}", hash, rel_path)
        },
        Algo::Blake3 => {
            let mut hasher = blake3::Hasher::new();

            loop {
                let read = reader.read(&mut buffer)?;
                if read == 0 { break; }
                hasher.update(&buffer[..read]);
            }

            let hash = format!("{}", hasher.finalize().to_hex());

            format!("blake3 {} {}", hash, rel_path)
        }
    };

    writeln!(temp_file, "{}", line)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::config::BUFFER_SIZE;

    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    fn run_process(content: &[u8], algo: Algo) -> String {
        let dir = tempfile::tempdir().expect("Failed to create temp dir");
        let file_path = dir.path().join("test_file.txt");
        let mut input_file = File::create(&file_path).expect("Failed to create input file");
        input_file.write_all(content).expect("Failed to write content");
        
        let mut output_file = NamedTempFile::new().expect("Failed to create temp output");
        let mut buffer = vec![0u8; BUFFER_SIZE];

        process_file(dir.path(), &file_path, &mut buffer, &algo, &mut output_file)
            .expect("process_file failed");

        std::fs::read_to_string(output_file.path())
            .expect("Failed to read output file")
            .trim()
            .to_string()
    }

    #[test]
    fn test_md5_empty_string() {
        let res = run_process(b"", Algo::Md5);
        assert_eq!(res, "md5 d41d8cd98f00b204e9800998ecf8427e test_file.txt");
    }

    #[test]
    fn test_md5_known_string() {
        let res = run_process(b"hello", Algo::Md5);
        assert_eq!(res, "md5 5d41402abc4b2a76b9719d911017c592 test_file.txt");
    }

    #[test]
    fn test_sha256_known_string() {
        let res = run_process(b"testing", Algo::Sha256);
        assert_eq!(res, "sha256 cf80cd8aed482d5d1527d7dc72fceff84e6326592848447d2dc0b0e87dfc9a90 test_file.txt");
    }

    #[test]
    fn test_sha256_empty_string() {
        let res = run_process(b"", Algo::Sha256);
        assert_eq!(res, "sha256 e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855 test_file.txt");
    }

    #[test]
    fn test_blake3_known_string() {
        let res = run_process(b"hola", Algo::Blake3);
        assert!(res.starts_with("blake3 "));
        let parts: Vec<&str> = res.split_whitespace().collect();
        assert_eq!(parts.len(), 3);
        assert_eq!(parts[0], "blake3");
        assert_eq!(parts[2], "test_file.txt");
        assert_eq!(parts[1].len(), 64);
        assert!(parts[1].chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn test_blake3_empty_string() {
        let res = run_process(b"", Algo::Blake3);
        let parts: Vec<&str> = res.split_whitespace().collect();
        assert_eq!(parts.len(), 3);
        assert_eq!(parts[0], "blake3");
        assert_eq!(parts[2], "test_file.txt");
        assert_eq!(parts[1].len(), 64);
        assert!(parts[1].chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn test_large_content() {
        let large_content = vec![b'A'; 2048];
        let res = run_process(&large_content, Algo::Sha256);
        assert!(res.contains("sha256"));
        assert!(res.contains("test_file.txt"));
        let parts: Vec<&str> = res.split_whitespace().collect();
        assert_eq!(parts[1].len(), 64);
    }

    #[test]
    fn test_small_buffer() {
        let dir = tempfile::tempdir().expect("Failed to create temp dir");
        let file_path = dir.path().join("test.txt");
        let mut input_file = File::create(&file_path).expect("Failed to create input file");
        input_file.write_all(b"test content").expect("Failed to write content");
        
        let mut output_file = NamedTempFile::new().expect("Failed to create temp output");
        let mut buffer = vec![0u8; 4];

        process_file(dir.path(), &file_path, &mut buffer, &Algo::Md5, &mut output_file)
            .expect("process_file failed");

        let result = std::fs::read_to_string(output_file.path())
            .expect("Failed to read output file");
        
        assert!(result.contains("md5"));
        assert!(result.contains("test.txt"));
        assert!(result.contains("9473fdd0d880a43c21b7778d34872157"));
    }

    #[test]
    fn test_output_format() {
        let res = run_process(b"sample", Algo::Sha256);
        assert!(res.matches(' ').count() == 2, "Should have exactly 2 spaces");
        let first_space = res.find(' ').unwrap();
        let second_space = res[first_space+1..].find(' ').unwrap() + first_space + 1;
        
        let algo_part = &res[..first_space];
        let hash_part = &res[first_space+1..second_space];
        let file_part = &res[second_space+1..];
        
        assert!(algo_part == "sha256" || algo_part == "md5" || algo_part == "blake3");
        assert!(!hash_part.is_empty());
        assert_eq!(file_part, "test_file.txt");
    }
}
