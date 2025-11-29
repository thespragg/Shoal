use super::{CommandExecutor, FileSystem, PathProvider};
use anyhow::{Result, anyhow};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

pub struct MockCommandExecutor {
    pub calls: Arc<std::sync::Mutex<Vec<(String, Vec<String>)>>>,
    pub should_fail: bool,
}

impl MockCommandExecutor {
    pub fn new() -> Self {
        Self {
            calls: Arc::new(std::sync::Mutex::new(Vec::new())),
            should_fail: false,
        }
    }

    pub fn with_failure(mut self, should_fail: bool) -> Self {
        self.should_fail = should_fail;
        self
    }
}

impl CommandExecutor for MockCommandExecutor {
    fn execute(&self, program: &str, args: &[&str]) -> Result<()> {
        let args_vec: Vec<String> = args.iter().map(|s| s.to_string()).collect();
        self.calls.lock().unwrap().push((program.to_string(), args_vec));

        if self.should_fail {
            Err(anyhow!("Mock command executor configured to fail"))
        } else {
            Ok(())
        }
    }
}

pub struct MockFileSystem {
    pub files: Arc<std::sync::Mutex<HashMap<PathBuf, String>>>,
    pub directories: Arc<std::sync::Mutex<HashMap<PathBuf, Vec<PathBuf>>>>,
}

impl MockFileSystem {
    pub fn new() -> Self {
        Self {
            files: Arc::new(std::sync::Mutex::new(HashMap::new())),
            directories: Arc::new(std::sync::Mutex::new(HashMap::new())),
        }
    }

    pub fn add_file(&self, path: PathBuf, content: String) {
        self.files.lock().unwrap().insert(path, content);
    }

    pub fn add_directory(&self, path: PathBuf, entries: Vec<PathBuf>) {
        self.directories.lock().unwrap().insert(path, entries);
    }
}

impl FileSystem for MockFileSystem {
    fn read_file(&self, path: &PathBuf) -> Result<String> {
        self.files
            .lock()
            .unwrap()
            .get(path)
            .cloned()
            .ok_or_else(|| anyhow!("File not found: {}", path.display()))
    }

    fn write_file(&self, path: &PathBuf, content: &str) -> Result<()> {
        self.files.lock().unwrap().insert(path.clone(), content.to_string());
        Ok(())
    }

    fn read_dir(&self, path: &PathBuf) -> Result<Vec<PathBuf>> {
        self.directories
            .lock()
            .unwrap()
            .get(path)
            .cloned()
            .ok_or_else(|| anyhow!("Directory not found: {}", path.display()))
    }

    fn create_dir_all(&self, _path: &PathBuf) -> Result<()> {
        Ok(())
    }

    fn exists(&self, path: &PathBuf) -> bool {
        self.files.lock().unwrap().contains_key(path)
            || self.directories.lock().unwrap().contains_key(path)
    }
}

pub struct MockPathProvider {
    pub current_dir: PathBuf,
    pub home_dir: PathBuf,
    pub data_local_dir: PathBuf,
}

impl MockPathProvider {
    pub fn new() -> Self {
        Self {
            current_dir: PathBuf::from("/test/current"),
            home_dir: PathBuf::from("/test/home"),
            data_local_dir: PathBuf::from("/test/data"),
        }
    }
}

impl PathProvider for MockPathProvider {
    fn current_dir(&self) -> Result<PathBuf> {
        Ok(self.current_dir.clone())
    }

    fn home_dir(&self) -> Result<PathBuf> {
        Ok(self.home_dir.clone())
    }

    fn data_local_dir(&self) -> Result<PathBuf> {
        Ok(self.data_local_dir.clone())
    }
}