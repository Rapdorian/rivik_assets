use crate::{formats::Format, Path};
use reerror::Result;

/// File format defintion for a byte buffer
pub struct Bin;

impl Format for Bin {
    type Output = Vec<u8>;

    fn parse(&self, path: &Path) -> Result<Self::Output> {
        let mut buffer = Vec::new();
        path.reader()?.read_to_end(&mut buffer)?;
        Ok(buffer)
    }
}
