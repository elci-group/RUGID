//! CAD Stress Test
//!
//! Tests the CAD parser and converter with various mesh complexities.

use rugid::cad::stl::parse_ascii_stl;
use rugid::cad::CADMesh;
use rugid::cad::converter::CADRenderer;
use rugid::geometry3d::{Point3D, Rotation3D};
use rugid::cell::CellId;
use std::time::Instant;
use std::io::Write;

fn main() {
    println!("🔬 RUGID CAD Stress Test");
    println!("{}", "=".repeat(60));
    
    // Test 1: Parse sample cube
    println!("\n📋 Test 1: Sample Cube (12 triangles)");
    test_sample_cube();
    
    // Test 2: Generate high-poly sphere
    println!("\n📋 Test 2: Procedural Sphere (varying polygon counts)");
    for subdivisions in [4, 8, 16, 32] {
        test_sphere(subdivisions);
    }
    
    // Test 3: Grid stress test
    println!("\n📋 Test 3: Grid Mesh (10,000+ triangles)");
    test_grid(100, 100);
    
    // Test 4: Render performance
    println!("\n📋 Test 4: Render Performance");
    test_render_performance();
    
    println!("\n✅ All stress tests passed!");
}

fn test_sample_cube() {
    let stl_content = rugid::cad::stl::generate_sample_stl();
    let temp_path = std::env::temp_dir().join("stress_cube.stl");
    {
        let mut file = std::fs::File::create(&temp_path).unwrap();
        file.write_all(stl_content.as_bytes()).unwrap();
    }
    
    let start = Instant::now();
    let mesh = parse_ascii_stl(&temp_path).unwrap();
    let parse_time = start.elapsed();
    
    println!("  ⏱️  Parse time: {:?}", parse_time);
    println!("  📐 Triangles: {}", mesh.triangles.len());
    println!("  📐 Vertices: {}", mesh.vertices.len());
    assert_eq!(mesh.triangles.len(), 12, "Cube should have 12 triangles");
}

fn test_sphere(subdivisions: usize) {
    let start = Instant::now();
    let mesh = generate_sphere(1.0, subdivisions);
    let gen_time = start.elapsed();
    
    let start = Instant::now();
    let _shape = mesh.to_rugid_shape();
    let convert_time = start.elapsed();
    
    println!("  🔵 Subdivisions: {} → {} triangles", 
        subdivisions, mesh.triangles.len());
    println!("     ⏱️  Generate: {:?} | Convert: {:?}", gen_time, convert_time);
}

fn test_grid(width: usize, height: usize) {
    let start = Instant::now();
    let mesh = generate_grid(width, height);
    let gen_time = start.elapsed();
    
    let start = Instant::now();
    let shape = mesh.to_rugid_shape();
    let convert_time = start.elapsed();
    
    println!("  📐 Grid: {}x{} → {} triangles", width, height, mesh.triangles.len());
    println!("  ⏱️  Generate: {:?} | Convert: {:?}", gen_time, convert_time);
    assert_eq!(shape.faces.len(), mesh.triangles.len());
}

fn test_render_performance() {
    let mesh = generate_sphere(1.0, 32);
    let mut cad_renderer = CADRenderer::new();
    let id = CellId::next();
    cad_renderer.register(id, &mesh);
    
    let rotation = Rotation3D::new(45.0, 30.0, 15.0);
    
    // Warm up
    for _ in 0..10 {
        let _ = cad_renderer.render(id, rotation, 400.0, 300.0, 200.0);
    }
    
    // Benchmark
    let iterations = 100;
    let start = Instant::now();
    for _ in 0..iterations {
        let _ = cad_renderer.render(id, rotation, 400.0, 300.0, 200.0);
    }
    let total_time = start.elapsed();
    let avg_time = total_time / iterations;
    let fps = 1.0 / avg_time.as_secs_f64();
    
    println!("  🎨 Render {} triangles x {} iterations", mesh.triangles.len(), iterations);
    println!("  ⏱️  Total: {:?} | Avg: {:?} | ~{:.0} FPS", total_time, avg_time, fps);
    
    // Target: at least 30 FPS for 2000+ triangle models
    assert!(fps > 30.0, "Render performance below 30 FPS!");
}

/// Generate a UV sphere mesh
fn generate_sphere(radius: f32, subdivisions: usize) -> CADMesh {
    use std::f32::consts::PI;
    
    let mut mesh = CADMesh::new(format!("sphere_{}", subdivisions));
    
    let lat_steps = subdivisions;
    let lon_steps = subdivisions * 2;
    
    for lat in 0..lat_steps {
        let theta1 = PI * lat as f32 / lat_steps as f32;
        let theta2 = PI * (lat + 1) as f32 / lat_steps as f32;
        
        for lon in 0..lon_steps {
            let phi1 = 2.0 * PI * lon as f32 / lon_steps as f32;
            let phi2 = 2.0 * PI * (lon + 1) as f32 / lon_steps as f32;
            
            // Four corners of quad
            let v1 = sphere_point(radius, theta1, phi1);
            let v2 = sphere_point(radius, theta1, phi2);
            let v3 = sphere_point(radius, theta2, phi1);
            let v4 = sphere_point(radius, theta2, phi2);
            
            // Calculate normals (for sphere, normal = normalized position)
            let n1 = v1.normalize();
            let n2 = v3.normalize();
            
            // Triangle 1
            if lat != 0 {
                mesh.add_triangle(v1, v2, v3, n1);
            }
            
            // Triangle 2
            if lat != lat_steps - 1 {
                mesh.add_triangle(v2, v4, v3, n2);
            }
        }
    }
    
    mesh.normalize();
    mesh
}

fn sphere_point(r: f32, theta: f32, phi: f32) -> Point3D {
    Point3D::new(
        r * theta.sin() * phi.cos(),
        r * theta.cos(),
        r * theta.sin() * phi.sin(),
    )
}

/// Generate a flat grid mesh
fn generate_grid(width: usize, height: usize) -> CADMesh {
    let mut mesh = CADMesh::new(format!("grid_{}x{}", width, height));
    
    let normal = Point3D::new(0.0, 1.0, 0.0);
    
    for x in 0..width {
        for z in 0..height {
            let x0 = x as f32;
            let z0 = z as f32;
            let x1 = x0 + 1.0;
            let z1 = z0 + 1.0;
            
            let v00 = Point3D::new(x0, 0.0, z0);
            let v10 = Point3D::new(x1, 0.0, z0);
            let v01 = Point3D::new(x0, 0.0, z1);
            let v11 = Point3D::new(x1, 0.0, z1);
            
            mesh.add_triangle(v00, v10, v01, normal);
            mesh.add_triangle(v10, v11, v01, normal);
        }
    }
    
    mesh.normalize();
    mesh
}
