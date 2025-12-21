//! STL File Parser
//!
//! Supports both ASCII and Binary STL formats.
//!
//! # ASCII Format
//! ```text
//! solid name
//!   facet normal ni nj nk
//!     outer loop
//!       vertex v1x v1y v1z
//!       vertex v2x v2y v2z
//!       vertex v3x v3y v3z
//!     endloop
//!   endfacet
//! endsolid name
//! ```
//!
//! # Binary Format
//! ```text
//! [80 bytes] Header
//! [4 bytes]  Number of triangles (u32 LE)
//! For each triangle:
//!   [12 bytes] Normal (3x f32 LE)
//!   [12 bytes] Vertex 1 (3x f32 LE)
//!   [12 bytes] Vertex 2 (3x f32 LE)
//!   [12 bytes] Vertex 3 (3x f32 LE)
//!   [2 bytes]  Attribute byte count
//! ```

use crate::geometry3d::Point3D;
use crate::cad::CADMesh;
use std::io::{Read, BufRead, BufReader};
use std::fs::File;
use std::path::Path;

/// Error type for STL parsing
#[derive(Debug)]
pub enum STLError {
    IoError(std::io::Error),
    ParseError(String),
    InvalidFormat(String),
}

impl From<std::io::Error> for STLError {
    fn from(err: std::io::Error) -> Self {
        STLError::IoError(err)
    }
}

impl std::fmt::Display for STLError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            STLError::IoError(e) => write!(f, "IO Error: {}", e),
            STLError::ParseError(s) => write!(f, "Parse Error: {}", s),
            STLError::InvalidFormat(s) => write!(f, "Invalid Format: {}", s),
        }
    }
}

/// Determine if an STL file is ASCII or Binary
fn is_ascii_stl(path: &Path) -> Result<bool, STLError> {
    let file = File::open(path)?;
    let mut reader = BufReader::new(file);
    let mut first_line = String::new();
    reader.read_line(&mut first_line)?;
    Ok(first_line.trim().to_lowercase().starts_with("solid"))
}

/// Parse an STL file (auto-detects ASCII vs Binary)
pub fn parse_stl(path: impl AsRef<Path>) -> Result<CADMesh, STLError> {
    let path = path.as_ref();
    
    if is_ascii_stl(path)? {
        parse_ascii_stl(path)
    } else {
        parse_binary_stl(path)
    }
}

/// Parse ASCII STL format
pub fn parse_ascii_stl(path: &Path) -> Result<CADMesh, STLError> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    
    let mut mesh = CADMesh::new(
        path.file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unnamed")
    );
    
    let mut current_normal = Point3D::new(0.0, 0.0, 0.0);
    let mut current_vertices: Vec<Point3D> = Vec::new();
    
    for line in reader.lines() {
        let line = line?;
        let line = line.trim().to_lowercase();
        
        if line.starts_with("solid") {
            // Extract name if present
            let name = line.strip_prefix("solid").unwrap_or("").trim();
            if !name.is_empty() {
                mesh.name = name.to_string();
            }
        } else if line.starts_with("facet normal") {
            // Parse normal: "facet normal nx ny nz"
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 5 {
                let nx = parts[2].parse::<f32>().unwrap_or(0.0);
                let ny = parts[3].parse::<f32>().unwrap_or(0.0);
                let nz = parts[4].parse::<f32>().unwrap_or(0.0);
                current_normal = Point3D::new(nx, ny, nz);
            }
        } else if line.starts_with("vertex") {
            // Parse vertex: "vertex x y z"
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 4 {
                let x = parts[1].parse::<f32>().unwrap_or(0.0);
                let y = parts[2].parse::<f32>().unwrap_or(0.0);
                let z = parts[3].parse::<f32>().unwrap_or(0.0);
                current_vertices.push(Point3D::new(x, y, z));
            }
        } else if line.starts_with("endfacet") {
            // Complete triangle
            if current_vertices.len() == 3 {
                mesh.add_triangle(
                    current_vertices[0],
                    current_vertices[1],
                    current_vertices[2],
                    current_normal,
                );
            }
            current_vertices.clear();
        }
    }
    
    Ok(mesh)
}

/// Parse Binary STL format
pub fn parse_binary_stl(path: &Path) -> Result<CADMesh, STLError> {
    let mut file = File::open(path)?;
    
    // Read 80-byte header
    let mut header = [0u8; 80];
    file.read_exact(&mut header)?;
    
    // Try to extract name from header (often contains model name)
    let header_str = String::from_utf8_lossy(&header);
    let name = header_str
        .trim_matches(char::from(0))
        .trim()
        .split_whitespace()
        .next()
        .unwrap_or("binary_stl")
        .to_string();
    
    let mut mesh = CADMesh::new(name);
    
    // Read triangle count (4 bytes, little-endian u32)
    let mut count_buf = [0u8; 4];
    file.read_exact(&mut count_buf)?;
    let triangle_count = u32::from_le_bytes(count_buf) as usize;
    
    // Read each triangle (50 bytes each)
    for _ in 0..triangle_count {
        // Normal (3x f32 = 12 bytes)
        let nx = read_f32_le(&mut file)?;
        let ny = read_f32_le(&mut file)?;
        let nz = read_f32_le(&mut file)?;
        let normal = Point3D::new(nx, ny, nz);
        
        // Vertex 1 (3x f32 = 12 bytes)
        let v1 = Point3D::new(
            read_f32_le(&mut file)?,
            read_f32_le(&mut file)?,
            read_f32_le(&mut file)?,
        );
        
        // Vertex 2 (3x f32 = 12 bytes)
        let v2 = Point3D::new(
            read_f32_le(&mut file)?,
            read_f32_le(&mut file)?,
            read_f32_le(&mut file)?,
        );
        
        // Vertex 3 (3x f32 = 12 bytes)
        let v3 = Point3D::new(
            read_f32_le(&mut file)?,
            read_f32_le(&mut file)?,
            read_f32_le(&mut file)?,
        );
        
        // Attribute byte count (2 bytes, usually 0, sometimes contains color)
        let mut attr_buf = [0u8; 2];
        file.read_exact(&mut attr_buf)?;
        
        mesh.add_triangle(v1, v2, v3, normal);
    }
    
    Ok(mesh)
}

/// Read a little-endian f32 from a reader
fn read_f32_le<R: Read>(reader: &mut R) -> Result<f32, STLError> {
    let mut buf = [0u8; 4];
    reader.read_exact(&mut buf)?;
    Ok(f32::from_le_bytes(buf))
}

/// Generate a sample STL file for testing (ASCII format)
pub fn generate_sample_stl() -> String {
    r#"solid sample_cube
  facet normal 0 0 -1
    outer loop
      vertex 0 0 0
      vertex 1 0 0
      vertex 1 1 0
    endloop
  endfacet
  facet normal 0 0 -1
    outer loop
      vertex 0 0 0
      vertex 1 1 0
      vertex 0 1 0
    endloop
  endfacet
  facet normal 0 0 1
    outer loop
      vertex 0 0 1
      vertex 1 1 1
      vertex 1 0 1
    endloop
  endfacet
  facet normal 0 0 1
    outer loop
      vertex 0 0 1
      vertex 0 1 1
      vertex 1 1 1
    endloop
  endfacet
  facet normal 0 -1 0
    outer loop
      vertex 0 0 0
      vertex 1 0 1
      vertex 1 0 0
    endloop
  endfacet
  facet normal 0 -1 0
    outer loop
      vertex 0 0 0
      vertex 0 0 1
      vertex 1 0 1
    endloop
  endfacet
  facet normal 1 0 0
    outer loop
      vertex 1 0 0
      vertex 1 0 1
      vertex 1 1 1
    endloop
  endfacet
  facet normal 1 0 0
    outer loop
      vertex 1 0 0
      vertex 1 1 1
      vertex 1 1 0
    endloop
  endfacet
  facet normal 0 1 0
    outer loop
      vertex 0 1 0
      vertex 1 1 0
      vertex 1 1 1
    endloop
  endfacet
  facet normal 0 1 0
    outer loop
      vertex 0 1 0
      vertex 1 1 1
      vertex 0 1 1
    endloop
  endfacet
  facet normal -1 0 0
    outer loop
      vertex 0 0 0
      vertex 0 1 0
      vertex 0 1 1
    endloop
  endfacet
  facet normal -1 0 0
    outer loop
      vertex 0 0 0
      vertex 0 1 1
      vertex 0 0 1
    endloop
  endfacet
endsolid sample_cube
"#.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_parse_ascii_stl() {
        let stl_content = generate_sample_stl();
        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(stl_content.as_bytes()).unwrap();
        
        let mesh = parse_stl(temp_file.path()).unwrap();
        
        assert_eq!(mesh.name, "sample_cube");
        assert_eq!(mesh.triangles.len(), 12); // A cube has 12 triangles
        assert_eq!(mesh.vertices.len(), 36);  // 12 triangles * 3 vertices
    }

    #[test]
    fn test_mesh_normalize() {
        let stl_content = generate_sample_stl();
        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(stl_content.as_bytes()).unwrap();
        
        let mut mesh = parse_stl(temp_file.path()).unwrap();
        mesh.normalize();
        
        // After normalization, max dimension should be ~1.0
        assert!(mesh.bounds.max_dimension() <= 1.01);
    }
}
