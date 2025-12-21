//! Generate a static SVG render of a complex CAD model

use rugid::cad::CADMesh;
use rugid::cad::converter::CADRenderer;
use rugid::geometry3d::{Point3D, Rotation3D};
use rugid::cell::CellId;
use std::f32::consts::PI;

fn main() {
    println!("🎨 Generating Complex CAD Render...");
    
    // Generate torus
    let mesh = generate_torus(1.0, 0.4, 48, 24);
    println!("{}", mesh.stats());
    
    // Create renderer
    let mut cad_renderer = CADRenderer::new();
    let shape_id = CellId::next();
    cad_renderer.register_colored(shape_id, &mesh, (180, 120, 60)); // Bronze
    
    // Render at a nice angle
    let rotation = Rotation3D::new(25.0, 45.0, 10.0);
    let shape_svg = cad_renderer
        .render(shape_id, rotation, 400.0, 300.0, 280.0)
        .unwrap_or_default();
    
    // Build full SVG
    let svg = format!(
        r##"<svg xmlns="http://www.w3.org/2000/svg" width="800" height="600" viewBox="0 0 800 600">
  <defs>
    <radialGradient id="bg" cx="50%" cy="50%" r="70%">
      <stop offset="0%" style="stop-color:#2c3e50"/>
      <stop offset="100%" style="stop-color:#0d1117"/>
    </radialGradient>
    <filter id="glow">
      <feGaussianBlur stdDeviation="2" result="blur"/>
      <feMerge>
        <feMergeNode in="blur"/>
        <feMergeNode in="SourceGraphic"/>
      </feMerge>
    </filter>
  </defs>
  <rect width="800" height="600" fill="url(#bg)" />
  <text x="400" y="40" text-anchor="middle" fill="#f1c40f" font-size="28" font-family="sans-serif" font-weight="bold" filter="url(#glow)">RUGID CAD Engine</text>
  <text x="400" y="70" text-anchor="middle" fill="#7f8c8d" font-size="16" font-family="sans-serif">Torus: {} triangles | Bronze Metallic</text>
  <g transform="translate(0, 20)">
    {}
  </g>
  <rect x="50" y="520" width="700" height="60" rx="5" fill="rgba(0,0,0,0.3)"/>
  <text x="400" y="545" text-anchor="middle" fill="#ecf0f1" font-size="14" font-family="monospace">Parameters: major_r=1.0 minor_r=0.4 segments=48x24</text>
  <text x="400" y="565" text-anchor="middle" fill="#95a5a6" font-size="12" font-family="monospace">Rotation: pitch=25 yaw=45 roll=10</text>
</svg>"##,
        mesh.triangles.len(),
        shape_svg
    );
    
    // Save to file
    let output_path = "/home/adminx/RUGID/cad_render_output.svg";
    std::fs::write(output_path, &svg).expect("Failed to write SVG");
    
    println!("✅ Saved to: {}", output_path);
    println!("📐 SVG size: {} bytes", svg.len());
}

fn generate_torus(major_radius: f32, minor_radius: f32, major_segments: usize, minor_segments: usize) -> CADMesh {
    let mut mesh = CADMesh::new("torus");
    
    for i in 0..major_segments {
        let theta1 = 2.0 * PI * i as f32 / major_segments as f32;
        let theta2 = 2.0 * PI * (i + 1) as f32 / major_segments as f32;
        
        for j in 0..minor_segments {
            let phi1 = 2.0 * PI * j as f32 / minor_segments as f32;
            let phi2 = 2.0 * PI * (j + 1) as f32 / minor_segments as f32;
            
            let v1 = torus_point(major_radius, minor_radius, theta1, phi1);
            let v2 = torus_point(major_radius, minor_radius, theta2, phi1);
            let v3 = torus_point(major_radius, minor_radius, theta1, phi2);
            let v4 = torus_point(major_radius, minor_radius, theta2, phi2);
            
            let n1 = torus_normal(theta1, phi1);
            let n2 = torus_normal(theta2, phi2);
            
            mesh.add_triangle(v1, v2, v3, n1);
            mesh.add_triangle(v2, v4, v3, n2);
        }
    }
    
    mesh.normalize();
    mesh
}

fn torus_point(major_r: f32, minor_r: f32, theta: f32, phi: f32) -> Point3D {
    Point3D::new(
        (major_r + minor_r * phi.cos()) * theta.cos(),
        minor_r * phi.sin(),
        (major_r + minor_r * phi.cos()) * theta.sin(),
    )
}

fn torus_normal(theta: f32, phi: f32) -> Point3D {
    Point3D::new(
        phi.cos() * theta.cos(),
        phi.sin(),
        phi.cos() * theta.sin(),
    ).normalize()
}
