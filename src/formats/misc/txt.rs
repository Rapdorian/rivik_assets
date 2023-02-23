use crate::{formats::Format, Path};
use reerror::Result;
/// File format defintion for a text file
pub struct Txt;

impl Format for Txt {
    type Output = String;

    fn parse(&self, path: &Path) -> Result<String> {
        let mut buffer = String::new();
        path.reader()?.read_to_string(&mut buffer)?;
        Ok(buffer)
    }
}
