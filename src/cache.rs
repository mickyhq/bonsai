use std::collections::{HashMap, HashSet};
use std::fs::{self, Metadata};
use std::path::{Path, PathBuf};
use std::time::UNIX_EPOCH;

use anyhow::{Context, Result};

use crate::parser::FileVariants;

const HEADER: &[u8] = b"BONSAI_PARSE_CACHE_V1\n";

#[derive(Debug)]
pub struct ParseCache {
    path: PathBuf,
    entries: HashMap<String, CacheEntry>,
    touched: HashSet<String>,
    dirty: bool,
}

#[derive(Debug, Clone)]
struct CacheEntry {
    size: u64,
    modified_ns: u128,
    variants: FileVariants,
}

impl ParseCache {
    pub fn load(path: PathBuf) -> Self {
        let entries = fs::read(&path)
            .ok()
            .and_then(|bytes| parse_cache_bytes(&bytes))
            .unwrap_or_default();

        Self {
            path,
            entries,
            touched: HashSet::new(),
            dirty: false,
        }
    }

    pub fn get(&mut self, path: &Path, metadata: &Metadata) -> Option<FileVariants> {
        let key = cache_key(path);
        self.touched.insert(key.clone());

        let entry = self.entries.get(&key)?;
        if entry.size == metadata.len() && entry.modified_ns == modified_ns(metadata) {
            return Some(entry.variants.clone());
        }

        None
    }

    pub fn put(&mut self, path: &Path, metadata: &Metadata, variants: FileVariants) {
        let key = cache_key(path);
        self.touched.insert(key.clone());
        self.entries.insert(
            key,
            CacheEntry {
                size: metadata.len(),
                modified_ns: modified_ns(metadata),
                variants,
            },
        );
        self.dirty = true;
    }

    pub fn retain_touched(&mut self) {
        let before = self.entries.len();
        self.entries.retain(|key, _| self.touched.contains(key));
        self.dirty |= self.entries.len() != before;
    }

    pub fn save(&self) -> Result<()> {
        if !self.dirty {
            return Ok(());
        }

        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("cannot create cache dir {}", parent.display()))?;
        }

        fs::write(&self.path, format_cache_bytes(&self.entries))
            .with_context(|| format!("cannot write parse cache {}", self.path.display()))
    }
}

pub fn cache_path_for_root(root: &Path) -> PathBuf {
    std::env::temp_dir()
        .join("bonsai-parse-cache")
        .join(format!("{:016x}.cache", stable_hash(root)))
}

fn stable_hash(path: &Path) -> u64 {
    let mut hash = 0xcbf29ce484222325;
    for byte in path.to_string_lossy().as_bytes() {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    hash
}

fn cache_key(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

fn modified_ns(metadata: &Metadata) -> u128 {
    metadata
        .modified()
        .ok()
        .and_then(|modified| modified.duration_since(UNIX_EPOCH).ok())
        .map(|duration| duration.as_nanos())
        .unwrap_or(0)
}

fn format_cache_bytes(entries: &HashMap<String, CacheEntry>) -> Vec<u8> {
    let mut output = HEADER.to_vec();
    let mut entries = entries.iter().collect::<Vec<_>>();
    entries.sort_by(|(left, _), (right, _)| left.cmp(right));

    for (path, entry) in entries {
        let full = entry.variants.full.as_deref();
        output.extend_from_slice(
            format!(
                "{} {} {} {} {} {}\n",
                path.len(),
                entry.modified_ns,
                entry.size,
                full.map(str::len)
                    .map(|length| length.to_string())
                    .unwrap_or_else(|| "-1".to_owned()),
                entry.variants.skeleton.len(),
                entry.variants.tree_map.len()
            )
            .as_bytes(),
        );
        push_field(&mut output, path);
        if let Some(full) = full {
            push_field(&mut output, full);
        }
        push_field(&mut output, &entry.variants.skeleton);
        push_field(&mut output, &entry.variants.tree_map);
    }

    output
}

fn push_field(output: &mut Vec<u8>, value: &str) {
    output.extend_from_slice(value.as_bytes());
    output.push(b'\n');
}

fn parse_cache_bytes(bytes: &[u8]) -> Option<HashMap<String, CacheEntry>> {
    if !bytes.starts_with(HEADER) {
        return None;
    }

    let mut cursor = HEADER.len();
    let mut entries = HashMap::new();

    while cursor < bytes.len() {
        let line = read_line(bytes, &mut cursor)?;
        if line.is_empty() {
            continue;
        }

        let parts = std::str::from_utf8(line)
            .ok()?
            .split_whitespace()
            .collect::<Vec<_>>();
        if parts.len() != 6 {
            return None;
        }

        let path_len = parts[0].parse::<usize>().ok()?;
        let modified_ns = parts[1].parse::<u128>().ok()?;
        let size = parts[2].parse::<u64>().ok()?;
        let full_len = parts[3].parse::<isize>().ok()?;
        let skeleton_len = parts[4].parse::<usize>().ok()?;
        let tree_map_len = parts[5].parse::<usize>().ok()?;

        let path = read_string(bytes, &mut cursor, path_len)?;
        let full = if full_len < 0 {
            None
        } else {
            Some(read_string(bytes, &mut cursor, full_len as usize)?)
        };
        let skeleton = read_string(bytes, &mut cursor, skeleton_len)?;
        let tree_map = read_string(bytes, &mut cursor, tree_map_len)?;

        entries.insert(
            path,
            CacheEntry {
                size,
                modified_ns,
                variants: FileVariants {
                    full,
                    skeleton,
                    tree_map,
                },
            },
        );
    }

    Some(entries)
}

fn read_line<'a>(bytes: &'a [u8], cursor: &mut usize) -> Option<&'a [u8]> {
    let start = *cursor;
    let rest = bytes.get(start..)?;
    let newline = rest.iter().position(|byte| *byte == b'\n')?;
    *cursor = start + newline + 1;
    Some(&rest[..newline])
}

fn read_string(bytes: &[u8], cursor: &mut usize, length: usize) -> Option<String> {
    let start = *cursor;
    let end = start.checked_add(length)?;
    let value = String::from_utf8(bytes.get(start..end)?.to_vec()).ok()?;
    if bytes.get(end) != Some(&b'\n') {
        return None;
    }
    *cursor = end + 1;
    Some(value)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trips_cache_entries() {
        let mut entries = HashMap::new();
        entries.insert(
            "/repo/src/lib.rs".to_owned(),
            CacheEntry {
                size: 12,
                modified_ns: 34,
                variants: FileVariants {
                    full: Some("fn main() {}\n".to_owned()),
                    skeleton: "fn main() { ... }\n".to_owned(),
                    tree_map: "fn main()\n".to_owned(),
                },
            },
        );

        let bytes = format_cache_bytes(&entries);
        let parsed = parse_cache_bytes(&bytes).unwrap();
        let entry = parsed.get("/repo/src/lib.rs").unwrap();

        assert_eq!(entry.size, 12);
        assert_eq!(entry.modified_ns, 34);
        assert_eq!(entry.variants.full.as_deref(), Some("fn main() {}\n"));
        assert_eq!(entry.variants.skeleton, "fn main() { ... }\n");
        assert_eq!(entry.variants.tree_map, "fn main()\n");
    }
}
