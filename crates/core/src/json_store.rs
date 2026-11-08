use std::{
    fs,
    io::{self, ErrorKind},
    path::{Path, PathBuf},
    sync::{Mutex, PoisonError},
};

use log::warn;
use serde::{Serialize, de::DeserializeOwned};

use crate::{
    config::get_data_directory,
    error::{ConfigurationError, Result},
};

pub struct JsonFileStore<Data> {
    path: PathBuf,
    data: Mutex<Data>,
}

impl<Data: Serialize + DeserializeOwned + Default> JsonFileStore<Data> {
    pub fn open(file_name: &str) -> Result<Self> {
        let directory = get_data_directory()?;
        fs::create_dir_all(&directory).map_err(|source| ConfigurationError::DataFile {
            operation: "create",
            path: directory.clone(),
            source,
        })?;
        let path = directory.join(file_name);
        let data = load_data(&path);
        Ok(Self {
            path,
            data: Mutex::new(data),
        })
    }

    pub fn is_persisted(&self) -> bool {
        self.path.exists()
    }

    pub fn read<Output>(&self, read: impl FnOnce(&Data) -> Output) -> Output {
        let data = self.data.lock().unwrap_or_else(PoisonError::into_inner);
        read(&data)
    }

    pub fn modify<Output>(&self, modify: impl FnOnce(&mut Data) -> Output) -> Output {
        let mut data = self.data.lock().unwrap_or_else(PoisonError::into_inner);
        let output = modify(&mut data);
        if let Err(error) = write_data(&self.path, &*data) {
            warn!("Failed to save {}: {error}", self.path.display());
        }
        output
    }
}

fn load_data<Data: DeserializeOwned + Default>(path: &Path) -> Data {
    match fs::read(path) {
        Ok(bytes) => serde_json::from_slice(&bytes).unwrap_or_else(|error| {
            warn!("Ignoring unreadable store {}: {error}", path.display());
            Data::default()
        }),
        Err(error) if error.kind() == ErrorKind::NotFound => Data::default(),
        Err(error) => {
            warn!("Failed to read store {}: {error}", path.display());
            Data::default()
        }
    }
}

fn write_data<Data: Serialize>(path: &Path, data: &Data) -> io::Result<()> {
    let bytes = serde_json::to_vec_pretty(data).map_err(io::Error::other)?;
    let temporary_path = path.with_extension("json.tmp");
    fs::write(&temporary_path, bytes)?;
    fs::rename(&temporary_path, path)
}
