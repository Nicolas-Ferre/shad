use crate::compilation::FILE_EXT;
use std::collections::HashMap;
use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::{fs, io};

pub(crate) fn read_files(
    source: impl SourceFolder,
) -> Result<HashMap<PathBuf, String>, Vec<(PathBuf, io::Error)>> {
    let mut errors = Vec::new();
    let files = source.parse(&mut errors);
    if errors.is_empty() {
        Ok(files.into_iter().collect())
    } else {
        Err(errors)
    }
}

/// A trait implemented for source folder accessors.
pub trait SourceFolder: Clone {
    /// Extract all Shad files.
    #[allow(private_interfaces)]
    fn parse(self, errors: &mut Vec<(PathBuf, io::Error)>) -> Vec<(PathBuf, String)>;

    /// Returns folder path.
    fn path(&self) -> PathBuf;
}

impl SourceFolder for &Path {
    #[allow(private_interfaces)]
    fn parse(self, errors: &mut Vec<(PathBuf, io::Error)>) -> Vec<(PathBuf, String)> {
        walkdir::WalkDir::new(self)
            .follow_links(true)
            .into_iter()
            .filter_map(|file| {
                match file {
                    Ok(file) => {
                        let is_file = !file.file_type().is_dir()
                            && file.path().extension() == Some(OsStr::new(FILE_EXT));
                        if is_file {
                            match fs::read_to_string(file.path()) {
                                Ok(code) => Some((file.path().into(), code)),
                                // coverage: off (not easy to test)
                                Err(error) => {
                                    errors.push((file.path().into(), error));
                                    None
                                } // coverage: on
                            }
                        } else {
                            None
                        }
                    }
                    Err(error) => {
                        if let Some(error) = error.into_io_error() {
                            errors.push((self.into(), error));
                        }
                        None
                    }
                }
            })
            .collect()
    }

    fn path(&self) -> PathBuf {
        self.into()
    }
}

// coverage: off (not used on native platforms)
impl SourceFolder for include_dir::Dir<'_> {
    #[allow(private_interfaces, clippy::only_used_in_recursion)]
    fn parse(self, errors: &mut Vec<(PathBuf, io::Error)>) -> Vec<(PathBuf, String)> {
        self.entries()
            .iter()
            .flat_map(|entry| match entry {
                include_dir::DirEntry::Dir(dir) => Self::parse(dir.clone(), errors),
                include_dir::DirEntry::File(file) => {
                    if file.path().extension() == Some(OsStr::new(FILE_EXT)) {
                        vec![(
                            file.path().into(),
                            String::from_utf8_lossy(file.contents()).into(),
                        )]
                    } else {
                        vec![]
                    }
                }
            })
            .collect()
    }

    fn path(&self) -> PathBuf {
        include_dir::Dir::path(self).into()
    }
}
// coverage: on
