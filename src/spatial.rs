use crate::scheduler::MegaCellId;

pub struct SpatialHasher {
    megacell_size: f32,
}

impl SpatialHasher {
    pub fn new(megacell_size: f32) -> Self {
        Self { megacell_size }
    }

    pub fn hash(&self, x: f32, y: f32) -> MegaCellId {
        let col = (x / self.megacell_size).floor() as i64;
        let row = (y / self.megacell_size).floor() as i64;
        
        // Simple packing: (row << 32) | col
        // We cast to u64 for the ID.
        // Note: This assumes positive coordinates mostly, or handles negative via 2's complement cast.
        // MegaCellId is u64.
        
        let col_u = col as u64;
        let row_u = row as u64;
        
        MegaCellId((row_u << 32) | (col_u & 0xFFFFFFFF))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spatial_hashing() {
        let hasher = SpatialHasher::new(100.0);

        let id1 = hasher.hash(50.0, 50.0);   // 0, 0
        let id2 = hasher.hash(150.0, 50.0);  // 1, 0
        let id3 = hasher.hash(50.0, 150.0);  // 0, 1
        let id4 = hasher.hash(150.0, 150.0); // 1, 1

        assert_ne!(id1, id2);
        assert_ne!(id1, id3);
        assert_ne!(id1, id4);
        
        // Check stability
        assert_eq!(id1, hasher.hash(10.0, 90.0));
    }
}
