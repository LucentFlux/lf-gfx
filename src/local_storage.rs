//! A key-value store implemented either via a file next to the executable, or
//! local browser storage on Web.

#[derive(Debug, thiserror::Error)]
pub enum StoreError {
    #[error("the file {} could not be written to: {}", .path.display(), .err)]
    FileUnwriteable {
        path: std::path::PathBuf,
        err: std::io::Error,
    },
    #[error("local browser storage was not available - ensure cookies are permitted for this site to store your game data, and then refresh the page")]
    LocalStorageUnavailable,
}

#[cfg(target_arch = "wasm32")]
fn load_web(key: &str) -> Option<String> {
    let storage = web_sys::window()
        .expect("app requires window")
        .local_storage()
        .ok()??;

    storage.get_item(key).ok()?
}

#[cfg(target_arch = "wasm32")]
fn store_web(key: &str, value: &str) -> Result<(), StoreError> {
    let storage = web_sys::window()
        .expect("app requires window")
        .local_storage()
        .map_err(|_| StoreError::LocalStorageUnavailable)?
        .ok_or(StoreError::LocalStorageUnavailable)?;

    storage
        .set_item(key, value)
        .map_err(|_| StoreError::LocalStorageUnavailable)
}

#[cfg(not(target_arch = "wasm32"))]
fn native_path(key: &str) -> std::path::PathBuf {
    use std::str::FromStr;

    const EXTENSION: &'static str = "lfx";
    assert!(
        key.is_ascii() && key.chars().all(|c| char::is_alphabetic(c) || c == '_'),
        "local storage keys must be alphanumeric/underscores only"
    );

    // Each directory to try, in order.
    let dir_list = [
        // Try next to the exe in a folder called `data`
        std::env::current_exe().ok().map(|dir| dir.join("data")),
        // Else try in the user's data dir in a folder named after the package
        #[cfg(not(debug_assertions))]
        dirs::data_dir().map(|dir| dir.join(env!("CARGO_PKG_NAME"))),
        // Else try the active dir
        std::env::current_dir().ok().map(|dir| dir.join("data")),
        // Else whatever `./data/` resolves to
        Some(std::path::PathBuf::from_str("./data/").expect("`./data/` is always valid dir")),
    ];

    for dir in dir_list {
        let dir = match dir {
            None => continue,
            Some(dir) => dir,
        };

        if !dir.is_dir() {
            if let Err(err) = std::fs::create_dir_all(&dir) {
                log::error!(
                    "failed to create data dir {}: {err}",
                    dir.canonicalize().unwrap_or_else(|_| dir.clone()).display()
                );
                continue;
            }
        }

        return dir.join(key).with_extension(EXTENSION);
    }

    panic!("no fallback directory was writeable");
}

#[cfg(not(target_arch = "wasm32"))]
fn load_native(key: &str) -> Option<String> {
    std::fs::read_to_string(native_path(key)).ok()
}

#[cfg(not(target_arch = "wasm32"))]
fn store_native(key: &str, value: &str) -> Result<(), StoreError> {
    let path = native_path(key);
    let res = std::fs::write(&path, value);
    if let Err(err) = res {
        log::error!("failed to record local value with key {key}: {err}");
        return Err(StoreError::FileUnwriteable { path, err });
    }
    return Ok(());
}

/// Loads the string associated with a key. These values persist through program
/// runs.
pub fn load(key: &str) -> Option<String> {
    #[cfg(target_arch = "wasm32")]
    return load_web(key);

    #[cfg(not(target_arch = "wasm32"))]
    return load_native(key);
}

/// Stores a string associated with a key. These values persist through program
/// runs.
pub fn store(key: &str, value: &str) -> Result<(), StoreError> {
    #[cfg(target_arch = "wasm32")]
    return store_web(key, value);

    #[cfg(not(target_arch = "wasm32"))]
    return store_native(key, value);
}
