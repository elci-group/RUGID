//! Complex Multi-Path Morphism Demo
//!
//! Demonstrates advanced shape morphing with adaptive path interpolation

use rugid::morphism::{ShapeType, shape_to_normalized_path, interpolate_paths_adaptive};
use rugid::svg::path_to_svg_d;

fn main() {
    println!("🔷 RUGID Complex Multi-Path Morphism Demo");
    println!("{}", "=".repeat(60));
    println!();

    // Demo 1: Triangle → Hexagon (3 points → 6 points)
    println!("📐 Demo 1: Polygon Complexity Transition");
    println!("{}", "-".repeat(60));
    
    let triangle = ShapeType::Polygon { sides: 3 };
    let hexagon = ShapeType::Polygon { sides: 6 };
    
    let tri_path = shape_to_normalized_path(&triangle, (0.0, 0.0, 100.0, 100.0), 3);
    let hex_path = shape_to_normalized_path(&hexagon, (0.0, 0.0, 100.0, 100.0), 6);
    
    println!("Triangle: {} vertices", tri_path.point_count());
    println!("Hexagon: {} vertices", hex_path.point_count());
    println!("\nMorphing stages:");
    
    for i in 0..=4 {
        let progress = i as f32 / 4.0;
        let morphed = interpolate_paths_adaptive(&tri_path, &hex_path, progress);
        println!("  {:.0}%: {} points", progress * 100.0, morphed.point_count());
    }
    
    // Demo 2: Circle → Square (different point distributions)
    println!("\n⭕ Demo 2: Circle to Rectangle");
    println!("{}", "-".repeat(60));
    
    let circle = ShapeType::Circle;
    let rect = ShapeType::Rectangle;
    
    let circle_path = shape_to_normalized_path(&circle, (0.0, 0.0, 100.0, 100.0), 16);
    let rect_path = shape_to_normalized_path(&rect, (0.0, 0.0, 100.0, 100.0), 16);
    
    println!("Morphing circle → rectangle with {} points", circle_path.point_count());
    
    for i in 0..=10 {
        let progress = i as f32 / 10.0;
        let morphed = interpolate_paths_adaptive(&circle_path, &rect_path, progress);
        
        let points: Vec<(f32, f32)> = morphed.points.iter().map(|p| (p.x, p.y)).collect();
        let svg_d = path_to_svg_d(&points, morphed.closed);
        
        if i % 2 == 0 {
            println!("  {:.0}%: {} chars of path data", progress * 100.0, svg_d.len());
        }
    }
    
    // Demo 3: Low-def → High-def (same shape, different resolutions)
    println!("\n🔍 Demo 3: Resolution Morphing");
    println!("{}", "-".repeat(60));
    
    let low_def_circle = shape_to_normalized_path(&circle, (0.0, 0.0, 100.0, 100.0), 8);
    let high_def_circle = shape_to_normalized_path(&circle, (0.0, 0.0, 100.0, 100.0), 32);
    
    println!("Low-def: {} points", low_def_circle.point_count());
    println!("High-def: {} points", high_def_circle.point_count());
    
    let morphed = interpolate_paths_adaptive(&low_def_circle, &high_def_circle, 0.5);
    println!("Morphed: {} points", morphed.point_count());
    
    // Demo 4: Complex polygon transitions
    println!("\n🎲 Demo 4: Polygon Ladder");
    println!("{}", "-".repeat(60));
    
    let shapes = vec![
        ("Triangle", 3),
        ("Square", 4),
        ("Pentagon", 5),
        ("Hexagon", 6),
        ("Octagon", 8),
        ("Dodecagon", 12),
    ];
    
    println!("Morphing through polygon sequence:\n");
    
    for window in shapes.windows(2) {
        let (from_name, from_sides) = window[0];
        let (to_name, to_sides) = window[1];
        
        let from_shape = ShapeType::Polygon { sides: from_sides };
        let to_shape = ShapeType::Polygon { sides: to_sides };
        
        let from_path = shape_to_normalized_path(&from_shape, (0.0, 0.0, 100.0, 100.0), from_sides as usize);
        let to_path = shape_to_normalized_path(&to_shape, (0.0, 0.0, 100.0, 100.0), to_sides as usize);
        
        println!("  {} ({} → {}) → {} ({} → {})",
            from_name, from_path.point_count(), to_path.point_count(),
            to_name, from_path.point_count(), to_path.point_count()
        );
        
        let halfway = interpolate_paths_adaptive(&from_path, &to_path, 0.5);
        println!("    Midpoint: {} points\n", halfway.point_count());
    }
    
    // Demo 5: Generate SVG animation frames
    println!("🎬 Demo 5: SVG Animation Frames");
    println!("{}", "-".repeat(60));
    
    let tri = ShapeType::Polygon { sides: 3 };
    let oct = ShapeType::Polygon { sides: 8 };
    
    let tri_path = shape_to_normalized_path(&tri, (50.0, 50.0, 100.0, 100.0), 3);
    let oct_path = shape_to_normalized_path(&oct, (50.0, 50.0, 100.0, 100.0), 8);
    
    let mut svg_frames = Vec::new();
    for i in 0..=5 {
        let progress = i as f32 / 5.0;
        let morphed = interpolate_paths_adaptive(&tri_path, &oct_path, progress);
        
        let points: Vec<(f32, f32)> = morphed.points.iter().map(|p| (p.x, p.y)).collect();
        let d = path_to_svg_d(&points, morphed.closed);
        
        let color_r = (progress * 255.0) as u8;
        let color_b = ((1.0 - progress) * 255.0) as u8;
        
        svg_frames.push(format!(
            r#"<path d="{}" fill="rgb({},100,{})" stroke="black" stroke-width="2" />"#,
            d, color_r, color_b
        ));
    }
    
    println!("Generated {} SVG frames for Triangle → Octagon", svg_frames.len());
    println!("Frame 0: {}", &svg_frames[0][..70.min(svg_frames[0].len())]);
    println!("Frame 5: {}", &svg_frames[5][..70.min(svg_frames[5].len())]);
    
    println!("\n✨ Demo complete!");
    println!("\n💡 Key Capabilities Demonstrated:");
    println!("  ✓ Adaptive interpolation handles different point counts");
    println!("  ✓ Triangle → Hexagon smooth morphing");
    println!("  ✓ Resolution morphing (8 points → 32 points)");
    println!("  ✓ Polygon ladder (3 → 4 → 5 → 6 → 8 → 12 sides)");
    println!("  ✓ SVG animation frame generation");
    println!("\n🎯 Complex multi-path morphing: COMPLETE");
}
