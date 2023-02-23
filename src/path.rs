use std::{
    fmt::Display,
    fs::File,
    io::{Read, Seek},
    path::PathBuf,
};

use reerror::{
    conversions::{invalid_argument, not_found, unimplemented},
    throw,
};
use uriparse::{Scheme, URIReference};

use crate::bin::BinRead;
use reerror::{Error, Result};

/// Path used to identify an asset
///
/// defaults to a file path if scheme is not specificed
#[derive(Debug, Hash, PartialEq, Eq)]
pub enum Path {
    /// Filesystem path
    /// Loads the asset from disk using the provided filepath
    File(PathBuf),
    /// Chunk
    /// Loads a chunk file from disk and optionally fetches a single chunk from that file
    /// # Example
    /// `bin:path/to/file.bin#CHUNKID`
    Chunk(PathBuf, Option<u128>),
}

impl TryFrom<&str> for Path {
    type Error = Error;

    fn try_from(path: &str) -> Result<Self> {
        let uri = throw!(URIReference::try_from(path).map_err(invalid_argument));

        // fetch an normalize scheme
        let mut scheme = uri.scheme().unwrap_or(&Scheme::File).clone();
        scheme.normalize();

        match scheme.as_ref() {
            "file" => Ok(Path::File(uri.path().to_string().into())),
            "bin" => Ok(Path::Chunk(
                uri.path().to_string().into(),
                throw!(uri
                    .fragment()
                    .map(|frag| u128::from_str_radix(frag, 16))
                    .transpose()
                    .map_err(invalid_argument)),
            )),
            scheme => Err(unimplemented(format!("Unsupported URI scheme {}", scheme))),
        }
    }
}

impl TryFrom<String> for Path {
    type Error = Error;

    fn try_from(value: String) -> Result<Self> {
        Self::try_from(value.as_str())
    }
}

impl TryFrom<&String> for Path {
    type Error = Error;

    fn try_from(value: &String) -> Result<Self> {
        Self::try_from(value.as_str())
    }
}

impl Display for Path {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Path::File(p) => match p.is_absolute() {
                true => write!(f, "file://{}", p.to_string_lossy()),
                false => write!(f, "file:{}", p.to_string_lossy()),
            },
            Path::Chunk(p, Some(id)) => match p.is_absolute() {
                true => write!(f, "bin://{}#{:X}", p.to_string_lossy(), id),
                false => write!(f, "bin:{}#{:X}", p.to_string_lossy(), id),
            },
            Path::Chunk(p, None) => match p.is_absolute() {
                true => write!(f, "bin://{}", p.to_string_lossy()),
                false => write!(f, "bin:{}", p.to_string_lossy()),
            },
        }
    }
}

pub(crate) trait AssetReader: Read + Seek {}
impl<T: Read + Seek> AssetReader for T {}

impl Path {
    pub(crate) fn reader(&self) -> Result<Box<dyn AssetReader>> {
        match self {
            Path::File(path) => Ok(Box::new(File::open(path)?)),
            Path::Chunk(path, Some(id)) => {
                // find chunk in file
                let mut file = File::open(path)?;
                while let Ok(chunk) = file.chunk() {
                    if chunk.id() == *id {
                        return Ok(Box::new(throw!(chunk.read(), "reading chunk")));
                    }
                }
                Err(not_found(format!("Chunk not found: {id:X}")))
            }
            _ => Err(unimplemented(
                format!("Unsupported path type: {}", self).as_str(),
            )),
        }
    }
}
