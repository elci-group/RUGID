/// Asset Management System - "The Vault"
///
/// A novel content-addressable asset management system for RUGID.
/// Assets are identified by their SHA-256 hash, enabling perfect caching,
/// deduplication, and dependency tracking.

pub mod registry;

pub use registry::{Asset, AssetHash, AssetRegistry, AssetError};
