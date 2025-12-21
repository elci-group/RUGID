//! Test program for RGPUM - renders a simple clear color
//! 
//! This validates that the Vulkan backend initializes correctly.

use rugid::platforms::rgpum::{RgpumPlatform, RgpumConfig};
use rugid::platform::Platform;
use winit::event::{Event, WindowEvent};
use winit::event_loop::EventLoop;
use winit::window::WindowBuilder;
use std::sync::Arc;

fn main() {
    println!("🚀 RGPUM Test - Custom Vulkan Wrapper POC");
    println!("{}", "=".repeat(60));
    
    let event_loop = EventLoop::new().unwrap();
    let window = WindowBuilder::new()
        .with_title("RGPUM Test - Black Window (Vulkan)")
        .with_inner_size(winit::dpi::LogicalSize::new(800, 600))
        .build(&event_loop)
        .unwrap();
    
    let window = Arc::new(window);
    
    let config = RgpumConfig::default();
    let mut platform = match RgpumPlatform::new(window.clone(), config) {
        Ok(p) => {
            println!("✅ RGPUM initialized successfully!");
            p
        }
        Err(e) => {
            eprintln!("❌ Failed to initialize RGPUM: {}", e);
            return;
        }
    };
    
    println!("\n📊 Platform Info:");
    let (width, height) = platform.dimensions();
    println!("  Dimensions: {}x{}", width, height);
    println!("\n💡 You should see a black window. Close to exit.");
    
    let _ = event_loop.run(move |event, elwt| {
        match event {
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::CloseRequested => {
                    println!("\n👋 Exiting...");
                    elwt.exit();
                }
                WindowEvent::RedrawRequested => {
                    // Render clear color (black)
                    platform.render(String::new());
                    window.request_redraw();
                }
                _ => {}
            },
            Event::AboutToWait => {
                window.request_redraw();
            }
            _ => {}
        }
    });
}
