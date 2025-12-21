use rugid::renderer::Renderer;
use rugid::cell::{CellId, Rotation3DState};
use rugid::shapes3d::ShapeType;
use rugid::geometry3d::{Point3D, Rotation3D};
use rugid::physics::light::{LightSource, OpticalMaterial};
use std::collections::HashMap;
use std::time::{Duration, Instant};

fn main() {
    println!("Starting Light Physics Demo - Multi-Angle Render...");
    
    let mut renderer = Renderer::new();
    
    // Add lights
    renderer.add_light(LightSource::point(
        Point3D::new(200.0, 200.0, -200.0),
        (255, 255, 255),
        2.0
    ));
    
    renderer.add_light(LightSource::point(
        Point3D::new(-200.0, -200.0, -200.0),
        (50, 50, 255),
        1.5
    ));

    // Scene Parameters
    let cube_z = 200.0;
    let cube_size = 200.0;
    let sphere_size = 40.0;
    let rel_pos = Point3D::new(40.0, 40.0, 40.0); // Relative to cube center
    
    // Angles to render
    let angles = vec![45.0, 135.0, 225.0];
    
    let cube_id = CellId::next();
    let sphere_id = CellId::next();
    
    for angle in angles {
        println!("Rendering angle: {} degrees", angle);
        
        // Register Cornell Box
        renderer.register_shape_3d(
            cube_id,
            None,
            ShapeType::CornellBox,
            0.0, 0.0, 800.0, 600.0, 
            cube_size,
            Rotation3DState::new(0.0, angle, 0.0)
        );
        renderer.register_motion_3d(
            cube_id,
            Some(rugid::motion3d::Translation3DState::new(0.0, 0.0, cube_z)),
            None
        );

        // Calculate Sphere Position
        // Rotate rel_pos by angle around Y
        let rad = angle.to_radians();
        let rx = rel_pos.x * rad.cos() + rel_pos.z * rad.sin();
        let ry = rel_pos.y;
        let rz = -rel_pos.x * rad.sin() + rel_pos.z * rad.cos();
        
        let sphere_pos = Point3D::new(rx, ry, cube_z + rz);

        // Register Sphere
        renderer.register_shape_3d(
            sphere_id,
            None,
            ShapeType::Sphere, 
            0.0, 0.0, 800.0, 600.0, 
            sphere_size,
            Rotation3DState::new(0.0, 0.0, 0.0) // No rotation for sphere
        );
        renderer.register_motion_3d(
            sphere_id,
            Some(rugid::motion3d::Translation3DState::new(sphere_pos.x, sphere_pos.y, sphere_pos.z)),
            None
        );
        
        // Update rotations (needed to apply the initial state to 'current')
        renderer.update_3d_rotations();

        let output = renderer.render_tick(HashMap::new(), None);
        
        let filename = format!("rendering_{}.svg", angle as i32);
        let path = format!("/home/adminx/.gemini/antigravity/brain/ffd0e548-18a6-41ed-aca7-f8242fd0bd02/{}", filename);
        
        // Inject background
        let svg_content = output.replacen(
            "<svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 800 600\" width=\"800\" height=\"600\">",
            "<svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 800 600\" width=\"800\" height=\"600\"><rect width=\"100%\" height=\"100%\" fill=\"#333333\"/>",
            1
        );
        
        std::fs::write(&path, &svg_content).expect("Unable to write file");
        println!("Saved {}", path);
    }
    
    println!("Multi-angle render completed.");
}
