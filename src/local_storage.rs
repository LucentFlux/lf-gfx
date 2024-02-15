//! A key-value store implemented either via a file next to the executable, or
//! local browser storage on Web.

#[cfg(target_arch = "wasm32")]
fn load_web(key: &str) -> Option<String> {
    let storage = web_sys::window().ok()?.local_storage().ok()?.ok()?;

    storage.get_item(key).ok()
}

#[cfg(target_arch = "wasm32")]
fn store_web(key: &str, value: &str) {
    let storage = web_sys::window().ok()?.local_storage().ok()?.ok()?;

    storage.set_item(key, value)
}

#[cfg(not(target_arch = "wasm32"))]
fn native_path(key: &str) -> std::path::PathBuf {
    use std::str::FromStr;

    const EXTENSION: &'static str = "lfx";
    assert!(
        key.is_ascii() && key.chars().all(|c| char::is_alphabetic(c) || c == '_'),
        "local storage keys must be alphanumeric/underscores only"
    );

    let fallback = std::path::PathBuf::from_str("./data/").unwrap();

    // Test for fallback path in case global path doesn't exist. Also allows for overriding
    if fallback.is_dir() {
        return fallback.join(key).with_extension(EXTENSION);
    }

    let mut path = dirs::document_dir()
        .map(|dir| dir.join("LucentFlux").join(env!("CARGO_PKG_NAME")))
        .unwrap_or_else(|| fallback.clone());

    if !path.is_dir() {
        if let Err(_) = std::fs::create_dir(&path) {
            path = fallback;
        }
    }

    return path.join(key).with_extension(EXTENSION);
}

#[cfg(not(target_arch = "wasm32"))]
fn load_native(key: &str) -> Option<String> {
    std::fs::read_to_string(native_path(key)).ok()
}

#[cfg(not(target_arch = "wasm32"))]
fn store_native(key: &str, value: &str) {
    let res = std::fs::write(native_path(key), value);
    if let Err(err) = res {
        log::error!("failed to record local value with key {key}: {err}")
    }
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
pub fn store(key: &str, value: &str) {
    #[cfg(target_arch = "wasm32")]
    store_web(key, value);

    #[cfg(not(target_arch = "wasm32"))]
    store_native(key, value);
}
