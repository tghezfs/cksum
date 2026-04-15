use std::path::Path;
use std::fs::{ self, Metadata};
use std::os::unix::fs::PermissionsExt;
use std::error::Error;

use tempfile::{Builder, NamedTempFile};

use crate::config::{FINAL_PREFIX, TEMP_PREFIX};

pub fn get_parent(metadata: Metadata, path: &Path) -> &Path {
    if metadata.is_file() {
        path
            .parent()
            .expect("Parent must be valid, because is a file, and a file is inside a dir")
    } else {
        path
    }
}

pub fn create_temp_file(parent: &Path) -> Result<NamedTempFile, Box<dyn Error>> {
    let temp_file = Builder::new()
        .prefix(TEMP_PREFIX)
        .tempfile_in(parent)?;

    Ok(temp_file)
}

pub fn finalize_checksum_file(temp: NamedTempFile, parent: &Path, timestamp: &str) -> Result<(), Box<dyn Error>> {
    let file_name = format!("{}{}.txt", FINAL_PREFIX, timestamp);    
    temp.persist(parent.join(&file_name))?;
    
    fs::set_permissions(parent.join(&file_name), fs::Permissions::from_mode(0o444))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;
    use tempfile::tempdir;

    #[test]
    fn test_get_parent_for_file() {
        let dir = tempdir().expect("Failed to create temp dir");
        let file_path = dir.path().join("test_file.txt");
        File::create(&file_path).expect("Failed to create file");

        let metadata = fs::metadata(&file_path).expect("Failed to get metadata");
        let parent = get_parent(metadata, &file_path);

        assert_eq!(parent, dir.path());
    }

    #[test]
    fn test_get_parent_for_dir() {
        let dir = tempdir().expect("Failed to create temp dir");
        let dir_path = dir.path();

        let metadata = fs::metadata(dir_path).expect("Failed to get metadata");
        let parent = get_parent(metadata, dir_path);

        assert_eq!(parent, dir_path);
    }

    #[test]
    fn test_create_temp_file_location() {
        let dir = tempdir().expect("Failed to create temp dir");
        let target_path = dir.path();

        let result = create_temp_file(target_path);
        assert!(result.is_ok());

        let named_temp = result.unwrap();
        let temp_path = named_temp.path();
        assert!(temp_path.exists());

        let parent_dir = temp_path.parent().expect("Temp file should have parent");
        assert_eq!(parent_dir, target_path);
    }

    #[test]
    fn test_create_temp_file_in_nonexistent_directory() {
        let non_existent_dir = Path::new("/nonexistent/dir/that/should/not/exist");
        let result = create_temp_file(non_existent_dir);
        assert!(result.is_err());
    }

    #[test]
    fn test_finalize_checksum_file() {
        let dir = tempdir().expect("Failed to create temp dir");
        
        let mut temp = Builder::new()
            .prefix(TEMP_PREFIX)
            .tempfile_in(dir.path())
            .expect("Failed to create manual temp");
        
        writeln!(temp, "md5 abc123 archivo.txt").expect("Failed to write temp");

        let timestamp = "20231027";
        let result = finalize_checksum_file(temp, dir.path(), timestamp);
        
        assert!(result.is_ok());

        let expected_name = format!("{}{}.txt", FINAL_PREFIX, timestamp);
        let final_path = dir.path().join(&expected_name);
        
        assert!(final_path.exists());
        
        let content = fs::read_to_string(&final_path).expect("Failed to read final file");
        assert!(content.contains("abc123"));

        #[cfg(unix)]
        {
            let metadata = fs::metadata(&final_path).expect("Metadata failed");
            let mode = metadata.permissions().mode();
            assert_eq!(mode & 0o777, 0o444, "Permissions should be read-only (444)");
        }
    }

    #[test]
    fn test_finalize_checksum_file_nonexistent_parent() {
        let temp = Builder::new()
            .prefix(TEMP_PREFIX)
            .tempfile()
            .expect("Failed to create temp");
        
        let non_existent_parent = Path::new("/nonexistent/dir");
        let timestamp = "20231027";
        
        let result = finalize_checksum_file(temp, non_existent_parent, timestamp);
        assert!(result.is_err());
    }
}
