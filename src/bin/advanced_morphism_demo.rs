//! Advanced Morphism Demo
//!
//! Demonstrates:
//! - SVG path rendering of morphed shapes
//! - Typewriter effect for text content morphing
//! - Combined shape + text morphing

use rugid::morphism::{ShapeType, shape_to_normalized_path, interpolate_paths};
use rugid::svg::path_to_svg_d;
use rugid::typewriter::{TypewriterEffect, morph_text_content, TextMorphMode};

fn main() {
    println!("🎨 RUGID Advanced Morphism Demo");
    println!("{}", "=".repeat(60));
    println!();

    // Demo 1: SVG Path Generation from Morphed Shapes
    println!("📐 Demo 1: SVG Path Generation");
    println!("{}", "-".repeat(60));
    
    let rect = ShapeType::Rectangle;
    let circle = ShapeType::Circle;
    
    // Generate normalized paths
    let rect_path = shape_to_normalized_path(&rect, (0.0, 0.0, 100.0, 100.0), 16);
    let circle_path = shape_to_normalized_path(&circle, (0.0, 0.0, 100.0, 100.0), 16);
    
    println!("Rectangle has {} points", rect_path.points.len());
    println!("Circle has {} points", circle_path.points.len());
    
    // Interpolate at 50%
    let morphed = interpolate_paths(&rect_path, &circle_path, 0.5);
    
    // Convert to SVG path data
    let points: Vec<(f32, f32)> = morphed.points.iter().map(|p| (p.x, p.y)).collect();
    let  svg_d = path_to_svg_d(&points, morphed.closed);
    
    println!("\nSVG path data (50% morph):");
    println!("{}", &svg_d[..80.min(svg_d.len())]); // Show first 80 chars
    println!("  ...(total {} chars)\n", svg_d.len());
    
    // Demo 2: Typewriter Effect
    println!("⌨️  Demo 2: Typewriter Effect");
    println!("{}", "-".repeat(60));
    
    let mut typewriter = TypewriterEffect::new("Hello, Morphism!".to_string());
    
    println!("Progressive reveal:");
    for _ in 0..18 {
        let visible = typewriter.tick();
        println!("  \"{}\"", visible);
        if typewriter.is_complete() {
            break;
        }
    }
    println!();
    
    // Demo 3: Text Morphing Modes
    println!("🔀 Demo 3: Text Morphing Modes");
    println!("{}", "-".repeat(60));
    
    let old_text = "Rectangle";
    let new_text = "Circle";
    
    println!("Typewriter mode:");
    for i in 0..=10 {
        let progress = i as f32 / 10.0;
        let result = morph_text_content(old_text, new_text, progress, TextMorphMode::Typewriter, None);
        println!("  {:.1}: \"{}\"", progress, result);
    }
    
    println!("\nCross-fade mode:");
    for i in 0..=10 {
        let progress = i as f32 / 10.0;
        let result = morph_text_content(old_text, new_text, progress, TextMorphMode::CrossFade, None);
        println!("  {:.1}: \"{}\"", progress, result);
    }
    
    println!("\nScramble mode:");
    for i in 0..=10 {
        let progress = i as f32 / 10.0;
        let result = morph_text_content(old_text, new_text, progress, TextMorphMode::Scramble, Some(42));
        println!("  {:.1}: \"{}\"", progress, result);
    }
    
    // Demo 4: Complete SVG Generation
    println!("\n🎨 Demo 4: Complete SVG Generation");
    println!("{}", "-".repeat(60));
    
    let svg_output = generate_morphing_svg();
    println!("Generated SVG with morphing shapes:");
    println!("{}", &svg_output[..200.min(svg_output.len())]);
    println!("  ...(total {} bytes)\n", svg_output.len());
    
    println!("✨ Demo complete!");
    println!("\n💡 Key Achievements:");
    println!("  ✓ SVG path generation from morphed shapes");
    println!("  ✓ Typewriter effect for progressive text reveal");
    println!("  ✓ Multiple text morphing modes");
    println!("  ✓ Full SVG output with transitioning shapes");
}

fn generate_morphing_svg() -> String {
    let mut output = String::new();
    
    // SVG header
    output.push_str(r#"<svg xmlns="http://www.w3.org/2000/svg" width="800" height="600" viewBox="0 0 800 600">"#);
    output.push('\n');
    
    // Generate 5 frames of morphing
    for i in 0..=4 {
        let progress = i as f32 / 4.0;
        let x_offset = i as f32 * 150.0 + 50.0;
        
        // Morph from rectangle to circle
        let rect = ShapeType::Rectangle;
        let circle = ShapeType::Circle;
        
        let rect_path = shape_to_normalized_path(&rect, (x_offset, 200.0, 100.0, 100.0), 24);
        let circle_path = shape_to_normalized_path(&circle, (x_offset, 200.0, 100.0, 100.0), 24);
        
        let morphed = interpolate_paths(&rect_path, &circle_path, progress);
        let points: Vec<(f32, f32)> = morphed.points.iter().map(|p| (p.x, p.y)).collect();
        let d = path_to_svg_d(&points, morphed.closed);
        
        // Color transition: blue to red
        let r = (progress * 255.0) as u8;
        let b = ((1.0 - progress) * 255.0) as u8;
        let color = format!("rgb({},{},{})", r, 0, b);
        
        // Add path element
        output.push_str(&format!(
            r#"  <path d="{}" fill="{}" stroke="black" stroke-width="2" />"#,
            d, color
        ));
        output.push('\n');
        
        // Add progress label
        output.push_str(&format!(
            r#"  <text x="{}" y="350" text-anchor="middle" font-size="14" fill="white">{}%</text>"#,
            x_offset + 50.0,
            (progress * 100.0) as u32
        ));
        output.push('\n');
    }
    
    output.push_str("</svg>");
    output
}
