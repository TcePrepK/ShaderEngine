use std::time::SystemTime;

pub struct FileWatcher {
    pub(crate) path: String,
    last_modified: SystemTime,
}

impl FileWatcher {
    pub fn new(path: String) -> Self {
        let last_modified = get_modified_time(&path);

        Self {
            path,
            last_modified,
        }
    }
}

impl FileWatcher {
    pub fn update(&mut self) -> bool {
        let current_modified = get_modified_time(&self.path);
        if current_modified > self.last_modified {
            self.last_modified = current_modified;
            true
        } else {
            false
        }
    }
}

fn get_modified_time(path: &str) -> SystemTime {
    let metadata = std::fs::metadata(path).unwrap();
    metadata.modified().unwrap()
}
