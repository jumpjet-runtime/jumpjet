//! `jumpjet:runtime/storage` host import as Rust `#[wasm_bindgen]` classes.
//!
//! OPFS is async on the main thread (its sync access handles are worker-only),
//! so this backs the storage VFS with synchronous `localStorage` — which fits the
//! sync WIT API and persists across reloads. Files are stored under a per-device
//! key prefix; bytes are stored as a string of code-point-per-byte (localStorage
//! is UTF-16, so bytes 0-255 round-trip), tagged `f:` (file) or `d:` (directory).

use js_sys::Array;
use wasm_bindgen::JsCast;
use wasm_bindgen::prelude::*;

fn ls() -> Option<web_sys::Storage> {
    web_sys::window()?.local_storage().ok().flatten()
}

fn encode_bytes(bytes: &[u8]) -> String {
    let mut s = String::from("f:");
    s.extend(bytes.iter().map(|b| *b as char));
    s
}
fn decode_bytes(value: &str) -> Option<Vec<u8>> {
    let rest = value.strip_prefix("f:")?;
    Some(rest.chars().map(|c| c as u8).collect())
}

/// Normalize a path: leading slash, no trailing slash, no empty segments.
fn normalize(path: &str) -> String {
    let parts: Vec<&str> = path
        .split('/')
        .filter(|p| !p.is_empty() && *p != ".")
        .collect();
    format!("/{}", parts.join("/"))
        .trim_end_matches('/')
        .to_owned()
}

#[wasm_bindgen]
pub struct StorageDevice {
    prefix: String,
}
#[wasm_bindgen]
impl StorageDevice {
    fn key(&self, path: &str) -> String {
        format!("{}{}", self.prefix, path)
    }

    fn ensure_parents(&self, path: &str) {
        if let Some(store) = ls() {
            let mut acc = String::new();
            for seg in path.split('/').filter(|s| !s.is_empty()) {
                acc.push('/');
                acc.push_str(seg);
                // stop before the final component (that's the file/dir itself)
                if acc == path {
                    break;
                }
                let _ = store.set_item(&self.key(&acc), "d:");
            }
        }
    }

    #[wasm_bindgen(js_name = createDir)]
    pub fn create_dir(&self, path: JsValue) {
        let p = path_str(&path);
        self.ensure_parents(&p);
        if let Some(store) = ls() {
            let _ = store.set_item(&self.key(&p), "d:");
        }
    }

    #[wasm_bindgen(js_name = listDir)]
    pub fn list_dir(&self, path: JsValue) -> JsValue {
        let dir = path_str(&path);
        let out = Array::new();
        if let Some(store) = ls() {
            let needle = if dir == "/" {
                format!("{}/", self.prefix)
            } else {
                format!("{}{}/", self.prefix, dir)
            };
            let mut seen = std::collections::HashSet::new();
            let len = store.length().unwrap_or(0);
            for i in 0..len {
                if let Ok(Some(k)) = store.key(i) {
                    if let Some(rest) = k.strip_prefix(&needle) {
                        // immediate child only
                        let child = rest.split('/').next().unwrap_or("");
                        if !child.is_empty() && seen.insert(child.to_owned()) {
                            let full = if dir == "/" {
                                format!("/{}", child)
                            } else {
                                format!("{}/{}", dir, child)
                            };
                            out.push(&JsValue::from(Path {
                                prefix: self.prefix.clone(),
                                path: full,
                            }));
                        }
                    }
                }
            }
        }
        out.into()
    }

    pub fn exists(&self, path: JsValue) -> bool {
        let p = path_str(&path);
        path_exists(&self.prefix, &p)
    }

    pub fn read(&self, path: JsValue) -> Option<Vec<u8>> {
        let p = path_str(&path);
        ls()?
            .get_item(&self.key(&p))
            .ok()
            .flatten()
            .and_then(|v| decode_bytes(&v))
    }

    #[wasm_bindgen(js_name = readString)]
    pub fn read_string(&self, path: JsValue) -> Option<String> {
        self.read(path).and_then(|b| String::from_utf8(b).ok())
    }

    pub fn write(&self, path: JsValue, content: JsValue) {
        let p = path_str(&path);
        let bytes = match get_str(&content, "tag").as_deref() {
            Some("string") => get_str(&content, "val").unwrap_or_default().into_bytes(),
            Some("bytes") => js_sys::Uint8Array::new(
                &js_sys::Reflect::get(&content, &JsValue::from_str("val"))
                    .unwrap_or(JsValue::UNDEFINED),
            )
            .to_vec(),
            _ => return,
        };
        self.ensure_parents(&p);
        if let Some(store) = ls() {
            let _ = store.set_item(&self.key(&p), &encode_bytes(&bytes));
        }
    }

    pub fn remove(&self, path: JsValue) -> Option<bool> {
        let p = path_str(&path);
        let store = ls()?;
        let key = self.key(&p);
        match store.get_item(&key).ok().flatten() {
            Some(v) if v.starts_with("f:") => {
                let _ = store.remove_item(&key);
                Some(true)
            }
            Some(_) | None => {
                // directory (explicit marker or implied by children)
                let needle = format!("{}/", key);
                let has_children = (0..store.length().unwrap_or(0)).any(|i| {
                    store
                        .key(i)
                        .ok()
                        .flatten()
                        .map(|k| k.starts_with(&needle))
                        .unwrap_or(false)
                });
                if has_children {
                    Some(false)
                } else if store.get_item(&key).ok().flatten().is_some() {
                    let _ = store.remove_item(&key);
                    Some(true)
                } else {
                    None
                }
            }
        }
    }
}

fn path_exists(prefix: &str, path: &str) -> bool {
    if let Some(store) = ls() {
        let key = format!("{}{}", prefix, path);
        if store.get_item(&key).ok().flatten().is_some() {
            return true;
        }
        // a directory may be implied by child keys
        let needle = format!("{}/", key);
        return (0..store.length().unwrap_or(0)).any(|i| {
            store
                .key(i)
                .ok()
                .flatten()
                .map(|k| k.starts_with(&needle))
                .unwrap_or(false)
        });
    }
    false
}
fn is_file(prefix: &str, path: &str) -> bool {
    ls().and_then(|s| s.get_item(&format!("{}{}", prefix, path)).ok().flatten())
        .map(|v| v.starts_with("f:"))
        .unwrap_or(false)
}

#[wasm_bindgen]
pub struct Path {
    prefix: String,
    path: String,
}
#[wasm_bindgen]
impl Path {
    #[wasm_bindgen(constructor)]
    pub fn new(storage: &StorageDevice, path: String) -> Path {
        Path {
            prefix: storage.prefix.clone(),
            path: normalize(&path),
        }
    }
    #[wasm_bindgen(js_name = toString)]
    pub fn to_string(&self) -> String {
        self.path.clone()
    }
    #[wasm_bindgen(js_name = isDir)]
    pub fn is_dir(&self) -> bool {
        path_exists(&self.prefix, &self.path) && !is_file(&self.prefix, &self.path)
    }
    #[wasm_bindgen(js_name = isFile)]
    pub fn is_file(&self) -> bool {
        is_file(&self.prefix, &self.path)
    }
    #[wasm_bindgen(js_name = isRoot)]
    pub fn is_root(&self) -> bool {
        self.path == "/" || self.path.is_empty()
    }
    pub fn extension(&self) -> Option<String> {
        let name = self.path.rsplit('/').next()?;
        name.rsplit_once('.').map(|(_, ext)| ext.to_owned())
    }
    pub fn filename(&self) -> Option<String> {
        self.path
            .rsplit('/')
            .next()
            .filter(|s| !s.is_empty())
            .map(|s| s.to_owned())
    }
    pub fn join(&self, path: String) -> Path {
        let joined = format!("{}/{}", self.path.trim_end_matches('/'), path);
        Path {
            prefix: self.prefix.clone(),
            path: normalize(&joined),
        }
    }
    pub fn parent(&self) -> Path {
        let parent = match self.path.rsplit_once('/') {
            Some((p, _)) if !p.is_empty() => p.to_owned(),
            _ => "/".to_owned(),
        };
        Path {
            prefix: self.prefix.clone(),
            path: parent,
        }
    }
}

fn get_str(o: &JsValue, k: &str) -> Option<String> {
    js_sys::Reflect::get(o, &JsValue::from_str(k))
        .ok()
        .and_then(|v| v.as_string())
}
/// Extract the path string from a `path` resource arg via its `toString` method.
fn path_str(p: &JsValue) -> String {
    let f = js_sys::Reflect::get(p, &JsValue::from_str("toString"))
        .ok()
        .and_then(|f| f.dyn_into::<js_sys::Function>().ok());
    match f {
        Some(f) => f
            .call0(p)
            .ok()
            .and_then(|v| v.as_string())
            .unwrap_or_default(),
        None => String::new(),
    }
}

// ---- free functions ----

#[wasm_bindgen(js_name = storageLocal)]
pub fn local() -> StorageDevice {
    StorageDevice {
        prefix: "jumpjet:local:".to_owned(),
    }
}
#[wasm_bindgen(js_name = storageCloud)]
pub fn cloud() -> Option<StorageDevice> {
    None
}
