use crate::{Result, internal, invalid};
use std::{
    fs::File,
    io::{self, Write},
    path::{Path, PathBuf},
};

pub struct Output {
    file: Option<tempfile::NamedTempFile>,
    path: Option<PathBuf>,
    overwrite: bool,
}
impl Output {
    pub fn new(path: Option<&Path>, overwrite: bool) -> Result<Self> {
        let path = path.filter(|path| *path != Path::new("-"));
        if let Some(path) = path {
            if path.exists() && !overwrite {
                return Err(invalid(format!(
                    "{} exists; pass --overwrite",
                    path.display()
                )));
            }
            let parent = path
                .parent()
                .filter(|path| !path.as_os_str().is_empty())
                .unwrap_or(Path::new("."));
            let file = tempfile::NamedTempFile::new_in(parent).map_err(internal)?;
            Ok(Self {
                file: Some(file),
                path: Some(path.to_owned()),
                overwrite,
            })
        } else {
            Ok(Self {
                file: None,
                path: None,
                overwrite,
            })
        }
    }
    pub fn write(&mut self, bytes: &[u8]) -> Result<()> {
        if let Some(file) = &mut self.file {
            file.write_all(bytes).map_err(internal)
        } else {
            io::stdout().lock().write_all(bytes).map_err(internal)
        }
    }
    pub fn finish(mut self) -> Result<()> {
        if let Some(mut file) = self.file.take() {
            file.flush().map_err(internal)?;
            let path = self.path.as_ref().unwrap();
            if self.overwrite {
                file.persist(path)
            } else {
                file.persist_noclobber(path)
            }
            .map_err(internal)?;
        } else {
            io::stdout().flush().map_err(internal)?;
        }
        Ok(())
    }
}
pub fn log_file(path: &Path, overwrite: bool) -> Result<File> {
    let mut options = File::options();
    options.write(true);
    if overwrite {
        options.create(true).truncate(true);
    } else {
        options.create_new(true);
    }
    options
        .open(path)
        .map_err(|e| invalid(format!("cannot open {}: {e}", path.display())))
}
