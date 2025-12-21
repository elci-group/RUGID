//! CAD File Import Module
//!
//! Provides parsing and conversion of CAD files (STL, OBJ) into RUGID assets.
//!
//! # Supported Formats
//! - STL (ASCII and Binary)
//! - OBJ (planned)
//! - glTF (planned)

pub mod stl;
pub mod converter;

use crate::geometry3d::Point3D;

/// Bounding box for a 3D mesh
#[derive(Clone, Debug)]
pub struct BoundingBox {
    pub min: Point3D,
    pub max: Point3D,
}

impl BoundingBox {
    pub fn new() -> Self {
        Self {
            min: Point3D::new(f32::MAX, f32::MAX, f32::MAX),
            max: Point3D::new(f32::MIN, f32::MIN, f32::MIN),
        }
    }

    pub fn expand(&mut self, point: &Point3D) {
        self.min.x = self.min.x.min(point.x);
        self.min.y = self.min.y.min(point.y);
        self.min.z = self.min.z.min(point.z);
        self.max.x = self.max.x.max(point.x);
        self.max.y = self.max.y.max(point.y);
        self.max.z = self.max.z.max(point.z);
    }

    pub fn center(&self) -> Point3D {
        Point3D::new(
            (self.min.x + self.max.x) / 2.0,
            (self.min.y + self.max.y) / 2.0,
            (self.min.z + self.max.z) / 2.0,
        )
    }

    pub fn size(&self) -> Point3D {
        Point3D::new(
            self.max.x - self.min.x,
            self.max.y - self.min.y,
            self.max.z - self.min.z,
        )
    }

    pub fn max_dimension(&self) -> f32 {
        let size = self.size();
        size.x.max(size.y).max(size.z)
    }
}

/// A triangle face with vertex indices
#[derive(Clone, Debug)]
pub struct Triangle {
    pub vertices: [usize; 3],
    pub normal: Point3D,
    pub color: Option<(u8, u8, u8)>,
}

/// A parsed CAD mesh
#[derive(Clone, Debug)]
pub struct CADMesh {
    pub name: String,
    pub vertices: Vec<Point3D>,
    pub triangles: Vec<Triangle>,
    pub bounds: BoundingBox,
}

impl CADMesh {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            vertices: Vec::new(),
            triangles: Vec::new(),
            bounds: BoundingBox::new(),
        }
    }

    /// Add a triangle with its three vertices
    pub fn add_triangle(&mut self, v0: Point3D, v1: Point3D, v2: Point3D, normal: Point3D) {
        let base_idx = self.vertices.len();
        
        self.vertices.push(v0);
        self.vertices.push(v1);
        self.vertices.push(v2);
        
        self.bounds.expand(&v0);
        self.bounds.expand(&v1);
        self.bounds.expand(&v2);
        
        self.triangles.push(Triangle {
            vertices: [base_idx, base_idx + 1, base_idx + 2],
            normal,
            color: None,
        });
    }

    /// Center the mesh at origin
    pub fn center_at_origin(&mut self) {
        let center = self.bounds.center();
        for v in &mut self.vertices {
            v.x -= center.x;
            v.y -= center.y;
            v.z -= center.z;
        }
        // Recalculate bounds
        self.bounds = BoundingBox::new();
        for v in &self.vertices {
            self.bounds.expand(v);
        }
    }

    /// Normalize mesh to fit in a unit cube (-0.5 to 0.5)
    pub fn normalize(&mut self) {
        self.center_at_origin();
        let scale = 1.0 / self.bounds.max_dimension();
        for v in &mut self.vertices {
            v.x *= scale;
            v.y *= scale;
            v.z *= scale;
        }
        // Recalculate bounds
        self.bounds = BoundingBox::new();
        for v in &self.vertices {
            self.bounds.expand(v);
        }
    }

    /// Get statistics about the mesh
    pub fn stats(&self) -> String {
        format!(
            "Mesh '{}': {} vertices, {} triangles, bounds: ({:.2}, {:.2}, {:.2}) to ({:.2}, {:.2}, {:.2})",
            self.name,
            self.vertices.len(),
            self.triangles.len(),
            self.bounds.min.x, self.bounds.min.y, self.bounds.min.z,
            self.bounds.max.x, self.bounds.max.y, self.bounds.max.z,
        )
    }
}

/// A scene containing multiple meshes
#[derive(Clone, Debug)]
pub struct CADScene {
    pub meshes: Vec<CADMesh>,
}

impl CADScene {
    pub fn new() -> Self {
        Self { meshes: Vec::new() }
    }

    pub fn add_mesh(&mut self, mesh: CADMesh) {
        self.meshes.push(mesh);
    }
}
