use crate::platform::{Platform, PlatformEvent};
use crate::input::InputEvent;
use winit::window::Window;
use winit::event::{Event, WindowEvent, ElementState, MouseButton};
use wgpu::util::DeviceExt;
use std::sync::Arc;
use glyphon::{
    Attrs, Buffer, Color, Family, FontSystem, Metrics, Resolution, Shaping, SwashCache, TextArea,
    TextAtlas, TextBounds, TextRenderer,
};

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct Vertex {
    position: [f32; 3],
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct PolygonVertex {
    position: [f32; 3],
    color: [f32; 4],
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct InstanceRaw {
    model: [[f32; 4]; 4],
    color: [f32; 4],
}

/// A text element to render
#[derive(Clone)]
pub struct TextElement {
    pub content: String,
    pub x: f32,
    pub y: f32,
    pub font_size: f32,
    pub color: [f32; 4],
    pub width: f32,
    pub height: f32,
    pub align: String, // "start", "middle", "end"
}

pub struct WgpuPlatform {
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    pub size: winit::dpi::PhysicalSize<u32>,
    render_pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    instance_buffer: wgpu::Buffer,
    instances: Vec<InstanceRaw>,
    // Polygon rendering
    polygon_pipeline: wgpu::RenderPipeline,
    polygon_buffer: wgpu::Buffer,
    polygon_vertices: Vec<PolygonVertex>,
    // Text rendering
    font_system: FontSystem,
    swash_cache: SwashCache,
    text_atlas: TextAtlas,
    text_renderer: TextRenderer,
    text_buffers: Vec<(Buffer, TextElement)>,
    // Event buffer
    events: Vec<PlatformEvent>,
    // Window reference
    window: Arc<Window>,
}

impl WgpuPlatform {
    pub async fn new(window: Arc<Window>) -> Self {
        let size = window.inner_size();

        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });
        
        let surface = unsafe { instance.create_surface_unsafe(wgpu::SurfaceTargetUnsafe::from_window(&*window).unwrap()) }.unwrap();

        let adapter = instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::default(),
            compatible_surface: Some(&surface),
            force_fallback_adapter: false,
        }).await.unwrap();

        let (device, queue) = adapter.request_device(
            &wgpu::DeviceDescriptor {
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::default(),
                label: None,
            },
            None,
        ).await.unwrap();

        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps.formats.iter()
            .copied()
            .find(|f| f.is_srgb())
            .unwrap_or(surface_caps.formats[0]);
            
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &config);

        // Shader for rectangles
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
        });

        // Pipeline for rectangles
        let render_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Render Pipeline Layout"),
            bind_group_layouts: &[],
            push_constant_ranges: &[],
        });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[
                    wgpu::VertexBufferLayout {
                        array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
                        step_mode: wgpu::VertexStepMode::Vertex,
                        attributes: &[
                            wgpu::VertexAttribute {
                                offset: 0,
                                shader_location: 0,
                                format: wgpu::VertexFormat::Float32x3,
                            }
                        ],
                    },
                    wgpu::VertexBufferLayout {
                        array_stride: std::mem::size_of::<InstanceRaw>() as wgpu::BufferAddress,
                        step_mode: wgpu::VertexStepMode::Instance,
                        attributes: &[
                            wgpu::VertexAttribute { offset: 0, shader_location: 5, format: wgpu::VertexFormat::Float32x4 },
                            wgpu::VertexAttribute { offset: 16, shader_location: 6, format: wgpu::VertexFormat::Float32x4 },
                            wgpu::VertexAttribute { offset: 32, shader_location: 7, format: wgpu::VertexFormat::Float32x4 },
                            wgpu::VertexAttribute { offset: 48, shader_location: 8, format: wgpu::VertexFormat::Float32x4 },
                            wgpu::VertexAttribute { offset: 64, shader_location: 9, format: wgpu::VertexFormat::Float32x4 },
                        ],
                    }
                ],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
        });

        // Unit Quad
        let vertices = &[
            Vertex { position: [0.0, 0.0, 0.0] },
            Vertex { position: [1.0, 0.0, 0.0] },
            Vertex { position: [1.0, 1.0, 0.0] },
            Vertex { position: [0.0, 0.0, 0.0] },
            Vertex { position: [1.0, 1.0, 0.0] },
            Vertex { position: [0.0, 1.0, 0.0] },
        ];

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let instance_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Instance Buffer"),
            contents: &[],
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        });

        let polygon_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Polygon Pipeline Layout"),
            bind_group_layouts: &[],
            push_constant_ranges: &[],
        });

        let polygon_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Polygon Pipeline"),
            layout: Some(&polygon_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_polygon",
                buffers: &[
                    wgpu::VertexBufferLayout {
                        array_stride: std::mem::size_of::<PolygonVertex>() as wgpu::BufferAddress,
                        step_mode: wgpu::VertexStepMode::Vertex,
                        attributes: &[
                            wgpu::VertexAttribute { offset: 0, shader_location: 0, format: wgpu::VertexFormat::Float32x3 },
                            wgpu::VertexAttribute { offset: 12, shader_location: 1, format: wgpu::VertexFormat::Float32x4 },
                        ],
                    }
                ],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
        });

        let polygon_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Polygon Buffer"),
            contents: &[],
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        });

        // Text rendering setup
        let mut font_system = FontSystem::new();
        println!("RUGID WGPU: Found {} fonts in system", font_system.db().faces().count());
        
        let swash_cache = SwashCache::new();
        let mut text_atlas = TextAtlas::new(&device, &queue, surface_format);
        let text_renderer = TextRenderer::new(
            &mut text_atlas,
            &device,
            wgpu::MultisampleState::default(),
            None,
        );

        Self {
            surface,
            device,
            queue,
            config,
            size,
            render_pipeline,
            vertex_buffer,
            instance_buffer,
            instances: Vec::new(),
            polygon_pipeline,
            polygon_buffer,
            polygon_vertices: Vec::new(),
            font_system,
            swash_cache,
            text_atlas,
            text_renderer,
            text_buffers: Vec::new(),
            events: Vec::new(),
            window,
        }
    }


    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
        }
    }
    
}

impl Platform for WgpuPlatform {
    fn poll_events(&mut self) -> Vec<PlatformEvent> {
        let events = self.events.clone();
        self.events.clear();
        events
    }

    fn handle_event(&mut self, event: &Event<()>) {
        match event {
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::Resized(physical_size) => {
                    self.resize(*physical_size);
                    self.events.push(PlatformEvent::Resize(physical_size.width as f32, physical_size.height as f32));
                }
                WindowEvent::CursorMoved { position, .. } => {
                    self.events.push(PlatformEvent::Input(InputEvent::PointerMove(position.x as f32, position.y as f32)));
                }
                WindowEvent::MouseInput { state, button, .. } => {
                    if *button == MouseButton::Left {
                        if *state == ElementState::Pressed {
                            self.events.push(PlatformEvent::Input(InputEvent::PointerDown));
                        } else {
                            self.events.push(PlatformEvent::Input(InputEvent::PointerUp));
                        }
                    }
                }
                _ => {}
            },
            _ => {}
        }
    }

    fn render(&mut self, svg: String) {
        // Parse rectangles
        self.instances.clear();
        
        let rects = SimpleSvgParser::parse_rects(&svg);
        
        for rect in rects {
            // Normalize using screen dimensions
            let screen_w = self.config.width as f32;
            let screen_h = self.config.height as f32;

            let sx = (rect.w / screen_w) * 2.0;
            let sy = (rect.h / screen_h) * 2.0;
            
            // Calculate center in NDC
            // Original x,y is top-left in pixels
            // Convert to NDC (-1 to 1)
            let left_ndc = (rect.x / screen_w) * 2.0 - 1.0;
            let top_ndc = 1.0 - (rect.y / screen_h) * 2.0; // Y is up in NDC
            
            // Center of the rect in NDC
            let cx = left_ndc + sx / 2.0;
            let cy = top_ndc - sy / 2.0; // Subtract because Y is up
            
            // Rotation
            let rad = rect.rotation.to_radians();
            let cos = rad.cos();
            let sin = rad.sin();
            
            // Matrix construction
            let m00 = sx * cos;
            let m01 = sx * sin;
            let m10 = -sy * sin;
            let m11 = sy * cos;
            
            let tx_final = cx - 0.5 * (m00 + m10);
            let ty_final = cy - 0.5 * (m01 + m11);

            self.instances.push(InstanceRaw {
                model: [
                    [m00, m01, 0.0, 0.0],
                    [m10, m11, 0.0, 0.0],
                    [0.0, 0.0, 1.0, 0.0],
                    [tx_final, ty_final, 0.0, 1.0],
                ],
                color: rect.fill,
            });
        }

        // Parse text elements
        let text_elements = SimpleSvgParser::parse_texts(&svg);
        
        // Prepare text buffers
        self.text_buffers.clear();
        for text_elem in text_elements {
            // Use resolved pixel size directly
            // The ontology resolver handles the relativistic scaling logic
            let pixel_size = text_elem.font_size;
            let pixel_height = pixel_size * 1.5; // Ensure enough vertical space
            
            let mut buffer = Buffer::new(
                &mut self.font_system, 
                Metrics::new(pixel_size, pixel_size * 1.2)
            );
            buffer.set_size(
                &mut self.font_system,
                self.size.width as f32, // Use full screen width to avoid wrapping for now
                pixel_height,
            );
            buffer.set_text(
                &mut self.font_system,
                &text_elem.content,
                Attrs::new().family(Family::SansSerif),
                Shaping::Advanced,
            );
            buffer.shape_until_scroll(&mut self.font_system);
            self.text_buffers.push((buffer, text_elem));
        }

        // Update instance buffer
        self.instance_buffer = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Instance Buffer"),
            contents: bytemuck::cast_slice(&self.instances),
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        });

        // Parse polygons
        let polygons = SimpleSvgParser::parse_polygons(&svg);
        self.polygon_vertices.clear();
        
        let screen_w = self.config.width as f32;
        let screen_h = self.config.height as f32;

        for poly in polygons {
            if poly.points.len() < 3 { continue; }
            
            // Triangle fan tessellation
            let p0 = poly.points[0];
            let ndc0 = [
                (p0.0 / screen_w) * 2.0 - 1.0,
                1.0 - (p0.1 / screen_h) * 2.0,
                0.0
            ];
            
            for i in 1..poly.points.len()-1 {
                let p1 = poly.points[i];
                let p2 = poly.points[i+1];
                
                let ndc1 = [
                    (p1.0 / screen_w) * 2.0 - 1.0,
                    1.0 - (p1.1 / screen_h) * 2.0,
                    0.0
                ];
                
                let ndc2 = [
                    (p2.0 / screen_w) * 2.0 - 1.0,
                    1.0 - (p2.1 / screen_h) * 2.0,
                    0.0
                ];
                
                self.polygon_vertices.push(PolygonVertex { position: ndc0, color: poly.fill });
                self.polygon_vertices.push(PolygonVertex { position: ndc1, color: poly.fill });
                self.polygon_vertices.push(PolygonVertex { position: ndc2, color: poly.fill });
            }
        }

        // Update polygon buffer
        if !self.polygon_vertices.is_empty() {
            self.polygon_buffer = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Polygon Buffer"),
                contents: bytemuck::cast_slice(&self.polygon_vertices),
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            });
        }

        // Get current texture
        let output = match self.surface.get_current_texture() {
            Ok(t) => t,
            Err(_) => return,
        };
        let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Render Encoder"),
        });

        // Draw geometry (Polygons + Rects)
        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color { r: 0.1, g: 0.1, b: 0.1, a: 1.0 }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
            });

            // Draw Polygons
            if !self.polygon_vertices.is_empty() {
                render_pass.set_pipeline(&self.polygon_pipeline);
                render_pass.set_vertex_buffer(0, self.polygon_buffer.slice(..));
                render_pass.draw(0..self.polygon_vertices.len() as u32, 0..1);
            }

            // Draw Rects
            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            render_pass.set_vertex_buffer(1, self.instance_buffer.slice(..));
            render_pass.draw(0..6, 0..self.instances.len() as u32);
        }

        // Prepare text areas for rendering
        let text_areas: Vec<TextArea> = self.text_buffers.iter().map(|(buffer, elem)| {
            // Measure text width to adjust alignment
            let mut width = 0.0f32;
            for run in buffer.layout_runs() {
                width = width.max(run.line_w);
            }
            
            let mut left = elem.x * self.size.width as f32 / 100.0;
            
            // Adjust based on text-anchor
            if elem.align == "middle" {
                left -= width / 2.0;
            } else if elem.align == "end" {
                left -= width;
            }
            
            TextArea {
                buffer,
                left,
                top: elem.y * self.size.height as f32 / 100.0,
                scale: 1.0,
                bounds: TextBounds {
                    left: 0,
                    top: 0,
                    right: self.size.width as i32,
                    bottom: self.size.height as i32,
                },
                default_color: Color::rgba(
                    (elem.color[0] * 255.0) as u8,
                    (elem.color[1] * 255.0) as u8,
                    (elem.color[2] * 255.0) as u8,
                    255,
                ),
            }
        }).collect();

        // Render text
        if !text_areas.is_empty() {
            let _ = self.text_renderer.prepare(
                &self.device,
                &self.queue,
                &mut self.font_system,
                &mut self.text_atlas,
                Resolution {
                    width: self.size.width,
                    height: self.size.height,
                },
                text_areas,
                &mut self.swash_cache,
            );

            {
                let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("Text Render Pass"),
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: &view,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Load,
                            store: wgpu::StoreOp::Store,
                        },
                    })],
                    depth_stencil_attachment: None,
                    occlusion_query_set: None,
                    timestamp_writes: None,
                });

                let _ = self.text_renderer.render(&self.text_atlas, &mut render_pass);
            }
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();
        
        self.window.request_redraw();
    }
}

struct SvgRect {
    x: f32, y: f32, w: f32, h: f32,
    fill: [f32; 4],
    rotation: f32,
}

struct SvgPolygon {
    points: Vec<(f32, f32)>,
    fill: [f32; 4],
}

struct SimpleSvgParser;

impl SimpleSvgParser {
    // ... existing extract methods ...

    fn parse_polygons(svg: &str) -> Vec<SvgPolygon> {
        let mut polygons = Vec::new();
        let mut remaining = svg;
        while let Some(start) = remaining.find("<polygon") {
            remaining = &remaining[start..];
            let end = remaining.find("/>").unwrap_or(remaining.len());
            let tag = &remaining[..end];
            
            let fill = Self::extract_fill(tag).unwrap_or([0.5, 0.5, 0.5, 1.0]);
            
            if let Some(points_str) = Self::extract_string_attr(tag, "points") {
                let mut points = Vec::new();
                for pair in points_str.split_whitespace() {
                    let coords: Vec<&str> = pair.split(',').collect();
                    if coords.len() == 2 {
                        if let (Ok(x), Ok(y)) = (coords[0].parse::<f32>(), coords[1].parse::<f32>()) {
                            points.push((x, y));
                        }
                    }
                }
                if !points.is_empty() {
                    polygons.push(SvgPolygon { points, fill });
                }
            }
            
            remaining = &remaining[end..];
        }
        polygons
    }
    fn extract_attr(tag: &str, key: &str) -> Option<f32> {
        let key_eq = format!("{}=\"", key);
        if let Some(start) = tag.find(&key_eq) {
            let rest = &tag[start + key_eq.len()..];
            if let Some(end) = rest.find('"') {
                let val = rest[..end].trim_end_matches("px");
                return val.parse().ok();
            }
        }
        None
    }

    fn extract_string_attr(tag: &str, key: &str) -> Option<String> {
        let key_eq = format!("{}=\"", key);
        if let Some(start) = tag.find(&key_eq) {
            let rest = &tag[start + key_eq.len()..];
            if let Some(end) = rest.find('"') {
                return Some(rest[..end].to_string());
            }
        }
        None
    }

    fn extract_fill(tag: &str) -> Option<[f32; 4]> {
        let mut alpha = 1.0;
        
        // Check for fill-opacity
        if let Some(val) = Self::extract_attr(tag, "fill-opacity") {
            alpha = val;
        }

        // fill="rgba(r,g,b,a)"
        if let Some(start) = tag.find("fill=\"rgba(") {
            let rest = &tag[start + 11..];
            if let Some(end) = rest.find(")") {
                let parts: Vec<&str> = rest[..end].split(',').collect();
                if parts.len() == 4 {
                    let r: f32 = parts[0].trim().parse().unwrap_or(0.0);
                    let g: f32 = parts[1].trim().parse().unwrap_or(0.0);
                    let b: f32 = parts[2].trim().parse().unwrap_or(0.0);
                    let a: f32 = parts[3].trim().parse().unwrap_or(1.0);
                    return Some([r/255.0, g/255.0, b/255.0, a * alpha]);
                }
            }
        }
        
        // fill="rgb(r,g,b)"
        if let Some(start) = tag.find("fill=\"rgb(") {
            let rest = &tag[start + 10..];
            if let Some(end) = rest.find(")") {
                let parts: Vec<&str> = rest[..end].split(',').collect();
                if parts.len() == 3 {
                    let r: f32 = parts[0].trim().parse().unwrap_or(0.0);
                    let g: f32 = parts[1].trim().parse().unwrap_or(0.0);
                    let b: f32 = parts[2].trim().parse().unwrap_or(0.0);
                    return Some([r/255.0, g/255.0, b/255.0, alpha]);
                }
            }
        }
        
        // fill="#RRGGBB"
        if let Some(start) = tag.find("fill=\"#") {
            let rest = &tag[start + 7..];
            if rest.len() >= 6 {
                let hex = &rest[..6];
                if let (Ok(r), Ok(g), Ok(b)) = (
                    u8::from_str_radix(&hex[0..2], 16),
                    u8::from_str_radix(&hex[2..4], 16),
                    u8::from_str_radix(&hex[4..6], 16),
                ) {
                    return Some([r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0, alpha]);
                }
            }
        }
        None
    }

    fn extract_rotation(tag: &str) -> f32 {
        if let Some(start) = tag.find("transform=\"rotate(") {
            let rest = &tag[start + 18..];
            if let Some(end) = rest.find(")") {
                let parts: Vec<&str> = rest[..end].split([',', ' ']).collect();
                if let Some(deg) = parts.first() {
                    return deg.parse().unwrap_or(0.0);
                }
            }
        }
        0.0
    }

    fn parse_rects(svg: &str) -> Vec<SvgRect> {
        let mut rects = Vec::new();
        let mut remaining = svg;
        while let Some(start) = remaining.find("<rect") {
            remaining = &remaining[start..];
            let end = remaining.find("/>").unwrap_or(remaining.len());
            let tag = &remaining[..end];
            
            let x = Self::extract_attr(tag, "x").unwrap_or(0.0);
            let y = Self::extract_attr(tag, "y").unwrap_or(0.0);
            let w = Self::extract_attr(tag, "width").unwrap_or(0.0);
            let h = Self::extract_attr(tag, "height").unwrap_or(0.0);
            let fill = Self::extract_fill(tag).unwrap_or([0.5, 0.5, 0.5, 1.0]);
            let rotation = Self::extract_rotation(tag);
            
            rects.push(SvgRect { x, y, w, h, fill, rotation });
            remaining = &remaining[end..];
        }
        rects
    }

    fn parse_texts(svg: &str) -> Vec<TextElement> {
        let mut texts = Vec::new();
        let mut remaining = svg;
        
        while let Some(start) = remaining.find("<text") {
            remaining = &remaining[start..];
            let tag_end = remaining.find(">").unwrap_or(remaining.len());
            let content_end = remaining.find("</text>").unwrap_or(remaining.len());
            
            let tag = &remaining[..tag_end];
            let content = if content_end > tag_end {
                &remaining[tag_end + 1..content_end]
            } else {
                ""
            };
            
            let x = Self::extract_attr(tag, "x").unwrap_or(0.0);
            let y = Self::extract_attr(tag, "y").unwrap_or(0.0);
            let font_size = Self::extract_attr(tag, "font-size").unwrap_or(16.0);
            let color = Self::extract_fill(tag).unwrap_or([1.0, 1.0, 1.0, 1.0]);
            let align = Self::extract_string_attr(tag, "text-anchor").unwrap_or("start".to_string());
            
            if !content.is_empty() {
                texts.push(TextElement {
                    content: content.to_string(),
                    x,
                    y,
                    font_size,
                    color,
                    width: 500.0, // Will be adjusted
                    height: font_size * 1.5,
                    align,
                });
            }
            
            remaining = &remaining[content_end.min(remaining.len())..];
        }
        
        texts
    }
}
