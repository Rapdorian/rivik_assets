//! Tomb asset manager
//!
//! This asset manager caches assets as they are loaded.
//! The cache is accessible from all threads but only grabs a thread safe handle once per thread.
//! Caches the threadsafe handle and creates thread local handles to the thread safe handle to
//! minimize atomic stores.
//!
//! # Usage
//!
//! ```
//! # use asset_inator::{load, formats::misc::Txt};
//! load("file:Cargo.toml", Txt).unwrap();
//! ```
//!
//! Some asset types will also need some parameters to be given to the parser. For example
//! ```
//! # use asset_inator::{load, formats::img::{ImageFormat, Img}};
//! load("file:examples/face.jpg", Img(ImageFormat::Jpeg)).unwrap();
//! ```

pub mod bin;
//pub mod handle;
mod mgr;
mod path;
pub use formats::Format;
pub use mgr::*;
pub use path::*;

/// File formats implementations
pub mod formats {
    use std::any::Any;

    use crate::Path;

    pub trait Asset: Any + Sync {}

    /// The `format` trait provides an interface for parsing a block of data into an asset
    pub trait Format {
        type Output;

        /// Parse a reader into some kind of asset
        fn parse(&self, r: &Path) -> reerror::Result<Self::Output>;
    }

    pub mod mesh;
    pub mod misc;
    pub mod img {
        mod general;
        pub use general::*;
    }
}
