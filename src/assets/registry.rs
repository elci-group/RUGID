/// Content-Addressable Asset Registry
///
/// Implements "The Vault" - a novel asset management system where assets
/// are identified by their content hash (SHA-256) rather than file paths.

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use sha2::{Sha256, Digest};

use crate::rdf::RdfDocument;

/// Content hash (SHA-256) used to uniquely identify assets
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct AssetHash([u8; 32]);

impl AssetHash {
    /// Compute hash from content
    pub fn from_bytes(content: &[u8]) -> Self {
        let mut hasher = Sha256::new();
        hasher.update(content);
        let result = hasher.finalize();
        let mut hash = [0u8; 32];
        hash.copy_from_slice(&result);
        AssetHash(hash)
    }

    /// Compute hash from string content
    pub fn from_str(content: &str) -> Self {
        Self::from_bytes(content.as_bytes())
    }

    /// Get hash as hex string
    pub fn to_hex(&self) -> String {
        self.0.iter()
            .map(|b| format!("{:02x}", b))
            .collect::<String>()
    }

    /// Parse from hex string
    pub fn from_hex(hex: &str) -> Result<Self, AssetError> {
        if hex.len() != 64 {
            return Err(AssetError::InvalidHash("Hash must be 64 hex characters".into()));
        }

        let mut hash = [0u8; 32];
        for i in 0..32 {
            let byte_str = &hex[i * 2..i * 2 + 2];
            hash[i] = u8::from_str_radix(byte_str, 16)
                .map_err(|_| AssetError::InvalidHash("Invalid hex character".into()))?;
        }

        Ok(AssetHash(hash))
    }
}

impl std::fmt::Display for AssetHash {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_hex())
    }
}

/// Asset types supported by the registry
#[derive(Debug, Clone)]
pub enum Asset {
    /// RDF scene/component document
    Rdf(RdfDocument),
    /// Raw SVG markup
    Svg(String),
    /// Generic text content
    Text(String),
}

impl Asset {
    /// Calculate the content hash of this asset
    pub fn hash(&self) -> AssetHash {
        match self {
            Asset::Rdf(doc) => {
                // Hash the serialized RDF
                // For now, we'll use a simplified approach
                // In production, we'd serialize the entire document
                AssetHash::from_str(&format!("{:?}", doc))
            }
            Asset::Svg(content) | Asset::Text(content) => {
                AssetHash::from_str(content)
            }
        }
    }
}

/// Errors that can occur during asset operations
#[derive(Debug, Clone)]
pub enum AssetError {
    NotFound(AssetHash),
    InvalidHash(String),
    IoError(String),
    ParseError(String),
}

impl std::fmt::Display for AssetError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AssetError::NotFound(hash) => write!(f, "Asset not found: {}", hash),
            AssetError::InvalidHash(msg) => write!(f, "Invalid hash: {}", msg),
            AssetError::IoError(msg) => write!(f, "I/O error: {}", msg),
            AssetError::ParseError(msg) => write!(f, "Parse error: {}", msg),
        }
    }
}

impl std::error::Error for AssetError {}

/// Content-addressable asset registry
pub struct AssetRegistry {
    /// Content-addressable storage: hash → asset
    store: HashMap<AssetHash, Arc<Asset>>,
    /// Path-to-hash mapping for convenience lookups
    path_index: HashMap<PathBuf, AssetHash>,
    /// Dependency graph: asset → list of dependency hashes
    dependencies: HashMap<AssetHash, Vec<AssetHash>>,
    /// Reverse dependency graph: asset → list of dependent hashes
    dependents: HashMap<AssetHash, Vec<AssetHash>>,
}

impl Default for AssetRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl AssetRegistry {
    pub fn new() -> Self {
        Self {
            store: HashMap::new(),
            path_index: HashMap::new(),
            dependencies: HashMap::new(),
            dependents: HashMap::new(),
        }
    }

    /// Insert an asset into the registry
    /// Returns the content hash
    pub fn insert(&mut self, asset: Asset) -> AssetHash {
        let hash = asset.hash();
        self.store.insert(hash, Arc::new(asset));
        hash
    }

    /// Insert an asset with an associated path
    pub fn insert_with_path(&mut self, asset: Asset, path: PathBuf) -> AssetHash {
        let hash = self.insert(asset);
        self.path_index.insert(path, hash);
        hash
    }

    /// Get an asset by its hash
    pub fn get(&self, hash: &AssetHash) -> Option<Arc<Asset>> {
        self.store.get(hash).cloned()
    }

    /// Get an asset by its path (if indexed)
    pub fn get_by_path(&self, path: &PathBuf) -> Option<Arc<Asset>> {
        self.path_index.get(path)
            .and_then(|hash| self.get(hash))
    }

    /// Get the hash associated with a path
    pub fn get_hash_by_path(&self, path: &PathBuf) -> Option<AssetHash> {
        self.path_index.get(path).copied()
    }

    /// Remove an asset from the registry
    pub fn remove(&mut self, hash: &AssetHash) -> Option<Arc<Asset>> {
        // Remove from main store
        let asset = self.store.remove(hash);

        // Remove from path index
        self.path_index.retain(|_, h| h != hash);

        // Remove from dependency graphs
        self.dependencies.remove(hash);
        self.dependents.remove(hash);

        asset
    }

    /// Register dependencies for an asset
    pub fn set_dependencies(&mut self, asset: AssetHash, deps: Vec<AssetHash>) {
        // Update forward dependencies
        self.dependencies.insert(asset, deps.clone());

        // Update reverse dependencies
        for dep in deps {
            self.dependents.entry(dep)
                .or_insert_with(Vec::new)
                .push(asset);
        }
    }

    /// Get dependencies of an asset
    pub fn get_dependencies(&self, hash: &AssetHash) -> Option<&[AssetHash]> {
        self.dependencies.get(hash).map(|v| v.as_slice())
    }

    /// Get dependents of an asset (assets that depend on this one)
    pub fn get_dependents(&self, hash: &AssetHash) -> Option<&[AssetHash]> {
        self.dependents.get(hash).map(|v| v.as_slice())
    }

    /// Get all assets that need to be reloaded if this asset changes
    pub fn get_cascade_reload(&self, hash: &AssetHash) -> Vec<AssetHash> {
        let mut to_reload = vec![*hash];
        let mut visited = std::collections::HashSet::new();
        visited.insert(*hash);

        let mut i = 0;
        while i < to_reload.len() {
            let current = to_reload[i];
            if let Some(deps) = self.get_dependents(&current) {
                for dep in deps {
                    if visited.insert(*dep) {
                        to_reload.push(*dep);
                    }
                }
            }
            i += 1;
        }

        to_reload
    }

    /// Get the total number of assets in the registry
    pub fn len(&self) -> usize {
        self.store.len()
    }

    /// Check if the registry is empty
    pub fn is_empty(&self) -> bool {
        self.store.is_empty()
    }

    /// Clear all assets from the registry
    pub fn clear(&mut self) {
        self.store.clear();
        self.path_index.clear();
        self.dependencies.clear();
        self.dependents.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_asset_hash() {
        let hash1 = AssetHash::from_str("Hello, World!");
        let hash2 = AssetHash::from_str("Hello, World!");
        let hash3 = AssetHash::from_str("Different content");

        assert_eq!(hash1, hash2);
        assert_ne!(hash1, hash3);
    }

    #[test]
    fn test_hash_hex_conversion() {
        let hash = AssetHash::from_str("test");
        let hex = hash.to_hex();
        let parsed = AssetHash::from_hex(&hex).unwrap();

        assert_eq!(hash, parsed);
    }

    #[test]
    fn test_registry_insert_get() {
        let mut registry = AssetRegistry::new();

        let content = "Test SVG content";
        let asset = Asset::Svg(content.to_string());
        let hash = registry.insert(asset);

        let retrieved = registry.get(&hash).unwrap();
        match retrieved.as_ref() {
            Asset::Svg(s) => assert_eq!(s, content),
            _ => panic!("Wrong asset type"),
        }
    }

    #[test]
    fn test_path_indexing() {
        let mut registry = AssetRegistry::new();

        let path = PathBuf::from("test.svg");
        let asset = Asset::Svg("content".to_string());
        let hash = registry.insert_with_path(asset, path.clone());

        let retrieved_hash = registry.get_hash_by_path(&path).unwrap();
        assert_eq!(hash, retrieved_hash);

        let retrieved_asset = registry.get_by_path(&path).unwrap();
        match retrieved_asset.as_ref() {
            Asset::Svg(s) => assert_eq!(s, "content"),
            _ => panic!("Wrong asset type"),
        }
    }

    #[test]
    fn test_dependency_tracking() {
        let mut registry = AssetRegistry::new();

        let dep1 = registry.insert(Asset::Text("dep1".into()));
        let dep2 = registry.insert(Asset::Text("dep2".into()));
        let parent = registry.insert(Asset::Text("parent".into()));

        registry.set_dependencies(parent, vec![dep1, dep2]);

        let deps = registry.get_dependencies(&parent).unwrap();
        assert_eq!(deps.len(), 2);
        assert!(deps.contains(&dep1));
        assert!(deps.contains(&dep2));

        let dependents = registry.get_dependents(&dep1).unwrap();
        assert_eq!(dependents.len(), 1);
        assert_eq!(dependents[0], parent);
    }

    #[test]
    fn test_cascade_reload() {
        let mut registry = AssetRegistry::new();

        // Create dependency chain: A -> B -> C
        let a = registry.insert(Asset::Text("A".into()));
        let b = registry.insert(Asset::Text("B".into()));
        let c = registry.insert(Asset::Text("C".into()));

        registry.set_dependencies(b, vec![a]);
        registry.set_dependencies(c, vec![b]);

        let cascade = registry.get_cascade_reload(&a);
        assert_eq!(cascade.len(), 3);
        assert!(cascade.contains(&a));
        assert!(cascade.contains(&b));
        assert!(cascade.contains(&c));
    }
}
