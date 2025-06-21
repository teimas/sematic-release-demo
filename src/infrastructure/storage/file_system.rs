//! File system storage implementations
//! 
//! Storage adapters that persist data to the file system using JSON, TOML,
//! or other file formats.

use std::path::Path;
use std::fs;
use serde::{Serialize, Deserialize};

/// Generic file system storage for serializable data
pub struct FileSystemStorage {
    base_path: std::path::PathBuf,
}

impl FileSystemStorage {
    pub fn new<P: AsRef<Path>>(base_path: P) -> std::io::Result<Self> {
        let path = base_path.as_ref().to_path_buf();
        fs::create_dir_all(&path)?;
        
        Ok(Self {
            base_path: path,
        })
    }
    
    pub async fn save_json<T: Serialize>(&self, filename: &str, data: &T) -> std::io::Result<()> {
        let file_path = self.base_path.join(format!("{}.json", filename));
        let json_data = serde_json::to_string_pretty(data)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        
        fs::write(file_path, json_data)
    }
    
    pub async fn load_json<T: for<'de> Deserialize<'de>>(&self, filename: &str) -> std::io::Result<Option<T>> {
        let file_path = self.base_path.join(format!("{}.json", filename));
        
        if !file_path.exists() {
            return Ok(None);
        }
        
        let json_data = fs::read_to_string(file_path)?;
        let data = serde_json::from_str(&json_data)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        
        Ok(Some(data))
    }
    
    pub async fn delete(&self, filename: &str) -> std::io::Result<bool> {
        let file_path = self.base_path.join(format!("{}.json", filename));
        
        if file_path.exists() {
            fs::remove_file(file_path)?;
            Ok(true)
        } else {
            Ok(false)
        }
    }
    
    pub async fn list_files(&self) -> std::io::Result<Vec<String>> {
        let entries = fs::read_dir(&self.base_path)?;
        let mut files = Vec::new();
        
        for entry in entries {
            let entry = entry?;
            if let Some(filename) = entry.file_name().to_str() {
                if filename.ends_with(".json") {
                    let name = filename.strip_suffix(".json").unwrap_or(filename);
                    files.push(name.to_string());
                }
            }
        }
        
        Ok(files)
    }
} 