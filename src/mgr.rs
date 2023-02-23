use std::{
    any::{type_name, Any, TypeId},
    cell::RefCell,
    collections::{hash_map::DefaultHasher, HashMap},
    hash::{Hash, Hasher},
    mem,
    rc::{self, Rc},
    sync::{self, Arc, RwLock},
};

use log::{info, trace};
use once_cell::{sync::Lazy, unsync};
use reerror::{conversions::invalid_argument, throw, Error, Result};

use crate::{formats::Format, path::Path};

static ASSET_CACHE: Lazy<RwLock<HashMap<u64, sync::Weak<dyn Any + Sync + Send>>>> =
    Lazy::new(|| RwLock::new(HashMap::new()));

thread_local! {
    #[allow(clippy::type_complexity)]
    static THREAD_ASSET_CACHE: unsync::Lazy<RefCell<HashMap<u64,rc::Weak<Arc<dyn Any + Sync + Send>>>>>  = unsync::Lazy::new(|| RefCell::new(HashMap::new()));
}

/// Load an asset from `path` it will load the asset using the file format specified by `format`
///
/// It will check a cache of previously loaded assets before loading the asset and if the asset
/// has not been cached it will cache the asset
pub fn load<F, P, E>(path: P, format: F) -> Result<Rc<Arc<F::Output>>>
where
    F: Format + Any,
    F::Output: Any + Send + Sync,
    P: TryInto<Path, Error = E>,
    E: Into<Error>,
{
    let path = match path.try_into() {
        Ok(path) => path,
        Err(e) => return Err(e.into()),
    };
    // generate a hash of the path and format
    let mut hash = DefaultHasher::new();
    path.hash(&mut hash);
    format.type_id().hash(&mut hash);
    let hash = hash.finish();

    let asset = THREAD_ASSET_CACHE.with(|cache| -> Result<_> {
        // check thread-local cache
        let key = cache.borrow().get(&hash).map(rc::Weak::clone);
        match key {
            Some(asset) => {
                trace!("found asset {} in thread cache", path);
                match asset.upgrade() {
                    Some(a) => Ok(a),
                    None => {
                        let asset = Rc::new(load_cache(hash, path, format)?);
                        cache.borrow_mut().insert(hash, Rc::downgrade(&asset));
                        Ok(asset)
                    }
                }
            }
            None => {
                // check global cache
                let asset = Rc::new(load_cache(hash, path, format)?);
                throw!(cache.try_borrow_mut()).insert(hash, Rc::downgrade(&asset));
                Ok(asset)
            }
        }
    })?;
    // this is some cursed shit
    // We need to manually implement downcast
    // first check if the types are the same
    if Arc::as_ref(&asset).type_id() != TypeId::of::<F::Output>() {
        return Err(invalid_argument(format!(
            "Expected didn't find asset of type: {}",
            type_name::<F::Output>()
        )));
    }

    // I'm 80% sure this is sound
    let typed =
        unsafe { mem::transmute::<Rc<Arc<dyn Any + Send + Sync>>, Rc<Arc<F::Output>>>(asset) };
    Ok(typed)
}

/// Attempt to load an asset from the global cache
fn load_cache<F>(hash: u64, path: Path, format: F) -> Result<Arc<dyn Any + Send + Sync>>
where
    F: Format,
    F::Output: Any + Send + Sync,
{
    let global_ro_cache = ASSET_CACHE.read().unwrap();
    match global_ro_cache.get(&hash) {
        Some(asset) => {
            trace!("found asset {} in global cache", path);
            match asset.upgrade() {
                Some(a) => Ok(a),
                None => {
                    mem::drop(global_ro_cache); // needed to allow a new write lock
                    insert_cache(hash, load_asset(path, format)?)
                }
            }
        }
        None => {
            // load asset from path
            info!("Loading asset: {}", path);
            mem::drop(global_ro_cache); // needed to allow a new write lock
            insert_cache(hash, load_asset(path, format)?)
        }
    }
}

/// Add an asset to the global cache
fn insert_cache<A: Any + Send + Sync>(hash: u64, asset: A) -> Result<Arc<dyn Any + Send + Sync>> {
    let asset: Arc<dyn Any + Send + Sync> = Arc::new(asset);
    ASSET_CACHE
        .write()
        .unwrap()
        .insert(hash, Arc::downgrade(&asset));
    Ok(asset)
}

/// Load an asset
/// doesn't handle the asset cache
fn load_asset<F>(path: Path, format: F) -> Result<F::Output>
where
    F: Format,
{
    Ok(throw!(
        format.parse(&path),
        "parsing asset {path} with format {:?}",
        std::any::type_name::<F>()
    ))
}
