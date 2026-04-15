use std::fs;
use std::io::{ Error, ErrorKind};
use std::path::Path;
use clap::Parser;
use walkdir::WalkDir;
use chrono::Local;

mod cli;
use cli::{Args, parse_algo};

mod hash;
use hash::process_file;

mod fs_op;
use fs_op::{ get_parent, create_temp_file, finalize_checksum_file };

mod config;
use config::{ BUFFER_SIZE, TEMP_PREFIX, FINAL_PREFIX};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Args = Args::parse();
    let path: &Path = Path::new(&args.path);
    let abs_path = fs::canonicalize(path)?; 

    let exists = abs_path.try_exists()?;
    if !exists {
        let error = Error::new(ErrorKind::NotFound, "Path Doesn't exists");

        return Err(Box::new(error));
    }

    let algo = parse_algo(&args.algorithm)?;
    let metadata = fs::metadata(&abs_path)?;
    let base_path: &Path = get_parent(metadata, &abs_path);
    let mut temp_file = create_temp_file(base_path)?;
    let mut buffer = [0; BUFFER_SIZE];

    for entry_res in WalkDir::new(&abs_path) {
        let entry = entry_res?;
        let entry_path = entry.path();
        let entry_metadata = entry.metadata()?;

        let name = entry_path
            .file_name()
            .expect("A file name always be valid, never can be: '.' or '..' or '/'")
            .to_str()
            .ok_or(Error::new(ErrorKind::InvalidData, "Filename not valid UTF-8"))?;

        if entry_metadata.is_file() && 
            !name.starts_with(FINAL_PREFIX) && 
            !name.starts_with(TEMP_PREFIX) {
            process_file(base_path, entry_path, &mut buffer, &algo, &mut temp_file)?;
        }
    }

    let timestamp: String = Local::now().format("%Y-%m-%d-%H%M%S").to_string();

    finalize_checksum_file(temp_file, base_path, &timestamp)
}
