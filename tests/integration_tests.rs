// tests/integration_test.rs
use std::os::unix::fs::PermissionsExt;
use std::fs;
use std::path::PathBuf;
use std::process::{Command, Output};
use std::thread;
use std::time::Duration;

use walkdir::WalkDir;

fn get_test_binary() -> Result<PathBuf, Box<dyn std::error::Error>> {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let project_name = std::env::var("CARGO_PKG_NAME")?;
    
    let candidate_paths = [
        format!("{}/target/debug/{}", manifest_dir, project_name),
        format!("{}/target/release/{}", manifest_dir, project_name),
    ];

    for candidate in &candidate_paths {
        if PathBuf::from(candidate).exists() && 
           PathBuf::from(candidate).is_file() {
            return Ok(PathBuf::from(candidate));
        }
    }

    Err(format!(
        "Binary not found. Run 'cargo build' first."
    ).into())
}

fn run_app(args: Vec<&str>) -> Result<Output, Box<dyn std::error::Error>> {
    let bin_path = get_test_binary()?;
    let mut cmd = Command::new(bin_path);
    cmd.args(&args);
    Ok(cmd.output()?)
}

fn wait_for_io() {
    thread::sleep(Duration::from_millis(150));
}

#[test]
fn test_main_sha256_valid_directory() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = tempfile::tempdir()?;
    let dir_path = temp_dir.path();
    let dummy_file = dir_path.join("archivo_prueba.txt");
    fs::write(&dummy_file, "contenido de prueba para sha256")?;

    let output = run_app(vec![
        "--path", dir_path.to_str().unwrap(), 
        "--algorithm", "sha256"
    ])?;

    assert!(output.status.success(), 
        "Execution failed.\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout), 
        String::from_utf8_lossy(&output.stderr));

    wait_for_io();

    let entries: Vec<_> = fs::read_dir(dir_path)?
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().map(|ft| ft.is_file()).unwrap_or(false))
        .map(|e| e.file_name().into_string().ok())
        .flatten()
        .collect();

    assert!(entries.len() >= 2, "Should have at least 2 files");

    let checksum_files: Vec<_> = entries.iter()
        .filter(|name| name.starts_with("checksums_") && name.ends_with(".txt"))
        .cloned()
        .collect();

    assert!(!checksum_files.is_empty(), "Checksum file must exist");

    let content = fs::read_to_string(&dir_path.join(&checksum_files[0]))?;
    assert!(content.starts_with("sha256"));
    
    let parts: Vec<&str> = content.split_whitespace().collect();
    assert_eq!(parts.len(), 3);
    assert_eq!(parts[1].len(), 64);
    assert!(parts[1].chars().all(|c| c.is_ascii_hexdigit()));

    Ok(())
}

#[test]
fn test_main_md5_valid_directory() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = tempfile::tempdir()?;
    let dir_path = temp_dir.path();
    let dummy_file = dir_path.join("prueba.md5.txt");
    fs::write(&dummy_file, "texto para hash md5")?;

    let output = run_app(vec![
        "--path", dir_path.to_str().unwrap(), 
        "--algorithm", "md5"
    ])?;

    assert!(output.status.success());
    wait_for_io();

    let entries: Vec<_> = fs::read_dir(dir_path)?
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().map(|ft| ft.is_file()).unwrap_or(false))
        .map(|e| e.file_name().into_string().ok())
        .flatten()
        .collect();

    let checksum_files: Vec<_> = entries.iter()
        .filter(|name| name.starts_with("checksums_") && name.ends_with(".txt"))
        .cloned()
        .collect();

    assert!(!checksum_files.is_empty());

    let content = fs::read_to_string(&dir_path.join(&checksum_files[0]))?;
    assert!(content.starts_with("md5"));

    let parts: Vec<&str> = content.split_whitespace().collect();
    assert_eq!(parts.len(), 3);
    assert_eq!(parts[1].len(), 32);
    assert!(parts[1].chars().all(|c| c.is_ascii_hexdigit()));

    Ok(())
}

#[test]
fn test_main_blake3_valid_directory() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = tempfile::tempdir()?;
    let dir_path = temp_dir.path();
    let dummy_file = dir_path.join("prueba.blake3.txt");
    fs::write(&dummy_file, "texto para hash blake3")?;

    let output = run_app(vec![
        "--path", dir_path.to_str().unwrap(), 
        "--algorithm", "blake3"
    ])?;

    assert!(output.status.success());
    wait_for_io();

    let entries: Vec<_> = fs::read_dir(dir_path)?
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().map(|ft| ft.is_file()).unwrap_or(false))
        .map(|e| e.file_name().into_string().ok())
        .flatten()
        .collect();

    let checksum_files: Vec<_> = entries.iter()
        .filter(|name| name.starts_with("checksums_") && name.ends_with(".txt"))
        .cloned()
        .collect();

    assert!(!checksum_files.is_empty());

    let content = fs::read_to_string(&dir_path.join(&checksum_files[0]))?;
    assert!(content.starts_with("blake3"));

    let parts: Vec<&str> = content.split_whitespace().collect();
    assert_eq!(parts.len(), 3);
    assert_eq!(parts[1].len(), 64);
    assert!(parts[1].chars().all(|c| c.is_ascii_hexdigit()));

    Ok(())
}

#[test]
fn test_fails_on_nonexistent_path() {
    let fake_path = "/tmp/no_existe_este_directorio_fake_xyz";

    match run_app(vec![
        "--path", fake_path, 
        "--algorithm", "sha256"
    ]) {
        Ok(output) => {
            assert!(!output.status.success());
            
            let stderr = String::from_utf8_lossy(&output.stderr);
            let stdout = String::from_utf8_lossy(&output.stdout);
            let combined = format!("{}{}", stderr, stdout);
            
            assert!(combined.contains("doesn't exist") || 
                    combined.contains("No such file") || 
                    combined.contains("NotFound") ||
                    combined.contains("Path Doesn't exist"));
        },
        Err(e) => panic!("Failed to run command: {}", e),
    }
}

#[test]
fn test_fails_on_invalid_algorithm() {
    let temp_dir = tempfile::tempdir().unwrap();

    let output = run_app(vec![
        "--path", temp_dir.path().to_str().unwrap(), 
        "--algorithm", "algoritmo_inexistente_123"
    ]).expect("Failed to run binary");

    assert!(!output.status.success());
    
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("Invalid Algorithm") || 
            stderr.contains("invalid") ||
            stderr.contains("Usage"));
}

#[test]
fn test_empty_directory_no_files() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = tempfile::tempdir()?;
    let dir_path = temp_dir.path();

    let output = run_app(vec![
        "--path", dir_path.to_str().unwrap(), 
        "--algorithm", "sha256"
    ])?;

    assert!(output.status.success());

    wait_for_io();

    let entries: Vec<_> = fs::read_dir(dir_path)?
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().map(|ft| ft.is_file()).unwrap_or(false))
        .map(|e| e.file_name().into_string().ok())
        .flatten()
        .collect();

    assert!(!entries.is_empty(), "Checksum file should be generated even for empty directory");

    Ok(())
}

#[test]
fn test_nested_directories_processed() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = tempfile::tempdir()?;
    let dir_path = temp_dir.path();
    
    let subdir1 = dir_path.join("folder1");
    let subdir2 = subdir1.join("subfolder2");
    fs::create_dir_all(&subdir2)?;
    
    fs::write(dir_path.join("root.txt"), "on root")?;
    fs::write(subdir1.join("level1.txt"), "on level1")?;
    fs::write(subdir2.join("level2.txt"), "en level2")?;

    let output = run_app(vec![
        "--path", dir_path.to_str().unwrap(), 
        "--algorithm", "sha256"
    ])?;

    assert!(output.status.success());
    wait_for_io();

    let entries: Vec<_> = WalkDir::new(dir_path)
        .min_depth(1)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
        .map(|e| e.file_name().to_string_lossy().to_string())
        .collect();

    assert!(entries.len() >= 4);

    let checksum_files: Vec<_> = entries.iter()
        .filter(|name| name.starts_with("checksums_") && name.ends_with(".txt"))
        .cloned()
        .collect();

    assert!(!checksum_files.is_empty());

    let content = fs::read_to_string(&dir_path.join(&checksum_files[0]))?;
    
    // Now checks for RELATIVE PATHS
    assert!(content.contains("root.txt"));
    assert!(content.contains("folder1/level1.txt"));
    assert!(content.contains("folder1/subfolder2/level2.txt"));

    Ok(())
}

#[test]
fn test_skip_temp_and_final_prefixes() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = tempfile::tempdir()?;
    let dir_path = temp_dir.path();
    
    fs::write(dir_path.join(".tmp-checksums_old.txt"), "old data")?;
    fs::write(dir_path.join("checksums_old.txt"), "previous ending")?;
    fs::write(dir_path.join("valid_normal.txt"), "true content")?;

    let output = run_app(vec![
        "--path", dir_path.to_str().unwrap(), 
        "--algorithm", "sha256"
    ])?;

    assert!(output.status.success());

    wait_for_io();

    let entries: Vec<_> = fs::read_dir(dir_path)?
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().map(|ft| ft.is_file()).unwrap_or(false))
        .map(|e| e.file_name().into_string().ok())
        .flatten()
        .collect();

    let checksum_files: Vec<_> = entries.iter()
        .filter(|name| name.starts_with("checksums_") && name.ends_with(".txt") && name != &"checksums_old.txt")
        .cloned()
        .collect();

    assert!(!checksum_files.is_empty());

    let content = fs::read_to_string(&dir_path.join(&checksum_files[0]))?;
    
    assert!(content.contains("valid_normal.txt"));
    
    // Old temp files should not appear in new checksum
    assert!(!content.contains(".tmp-checksums_old"));
    assert!(!content.contains("checksums_old"));

    Ok(())
}

#[test]
fn test_case_insensitive_algorithm_input() {
    let temp_dir = tempfile::tempdir().unwrap();
    let dir_path = temp_dir.path();
    fs::write(dir_path.join("test.txt"), "contenido").unwrap();

    let algorithms = vec!["sha256", "SHA256", "Sha256"];
    
    for algo in algorithms {
        let output = run_app(vec![
            "--path", dir_path.to_str().unwrap(), 
            "--algorithm", algo
        ]).expect("Failed to run");
        
        assert!(output.status.success(), 
            "Algorithm '{}' should be case-insensitive", algo);
    }
}

#[test]
fn test_file_permissions_on_checksum() -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(unix)]
    {
        let temp_dir = tempfile::tempdir()?;
        let dir_path = temp_dir.path();
        fs::write(dir_path.join("permiso_test.txt"), "data")?;

        let output = run_app(vec![
            "--path", dir_path.to_str().unwrap(), 
            "--algorithm", "sha256"
        ])?;

        assert!(output.status.success());

        wait_for_io();

        let entries: Vec<_> = fs::read_dir(dir_path)?
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().map(|ft| ft.is_file()).unwrap_or(false))
            .collect();

        let checksum_file = entries.into_iter()
            .find(|e| {
                let file_name = e.file_name();
                let name = file_name.to_string_lossy();
                name.starts_with("checksums_") && name.ends_with(".txt")
            })
            .expect("Checksum not found");

        let path = checksum_file.path();
        let metadata = fs::metadata(&path)?;
        let mode = metadata.permissions().mode();
        
        assert_eq!(mode & 0o777, 0o444, 
            "Invalid permissions. Expected 0o444, got {:o}", mode & 0o777);
    }

    #[cfg(not(unix))]
    {
        println!("SKIP: Permission test only applies on Unix");
    }

    Ok(())
}
