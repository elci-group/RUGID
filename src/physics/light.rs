use crate::geometry3d::Point3D;

/// A ray of light in 3D space
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Ray {
    pub origin: Point3D,
    pub direction: Point3D,
}

impl Ray {
    pub fn new(origin: Point3D, direction: Point3D) -> Self {
        Self {
            origin,
            direction: direction.normalize(),
        }
    }

    pub fn at(&self, t: f32) -> Point3D {
        Point3D::new(
            self.origin.x + self.direction.x * t,
            self.origin.y + self.direction.y * t,
            self.origin.z + self.direction.z * t,
        )
    }
}

/// Types of textures
#[derive(Clone, Debug, PartialEq)]
pub enum TextureType {
    None,
    Checkerboard { color1: (u8, u8, u8), color2: (u8, u8, u8), scale: f32 },
    Image { path: String, opacity: f32 },
}

/// Optical properties of a material
#[derive(Clone, Debug, PartialEq)]
pub struct OpticalMaterial {
    /// Base color (RGB)
    pub color: (u8, u8, u8),
    /// Ambient reflection coefficient (0.0 - 1.0)
    pub ambient: f32,
    /// Diffuse reflection coefficient (0.0 - 1.0)
    pub diffuse: f32,
    /// Specular reflection coefficient (0.0 - 1.0)
    pub specular: f32,
    /// Shininess exponent for specular highlights (higher = smaller, sharper highlight)
    pub shininess: f32,
    /// Reflectivity (0.0 = matte, 1.0 = mirror)
    pub reflectivity: f32,
    /// Transparency (0.0 = opaque, 1.0 = fully transparent)
    pub transparency: f32,
    /// Refractive index (e.g., 1.0 for air, 1.5 for glass, 2.4 for diamond)
    pub refractive_index: f32,
    /// Texture mapping
    pub texture: TextureType,
    /// Surface type for rendering optimization
    pub surface: SurfaceType,
}

#[derive(Clone, Debug, PartialEq)]
pub enum SurfaceType {
    Matte,
    Plastic,
    Metal,
    Glass,
}

impl Default for OpticalMaterial {
    fn default() -> Self {
        Self {
            color: (200, 200, 200),
            ambient: 0.1,
            diffuse: 0.7,
            specular: 0.2,
            shininess: 32.0,
            reflectivity: 0.0,
            transparency: 0.0,
            refractive_index: 1.0,
            texture: TextureType::None,
            surface: SurfaceType::Matte,
        }
    }
}

impl OpticalMaterial {
    pub fn matte(color: (u8, u8, u8)) -> Self {
        Self {
            color,
            ambient: 0.1,
            diffuse: 0.9,
            specular: 0.0,
            shininess: 1.0,
            surface: SurfaceType::Matte,
            ..Default::default()
        }
    }

    pub fn plastic(color: (u8, u8, u8)) -> Self {
        Self {
            color,
            ambient: 0.1,
            diffuse: 0.7,
            specular: 0.4,
            shininess: 32.0,
            surface: SurfaceType::Plastic,
            ..Default::default()
        }
    }

    pub fn metal(color: (u8, u8, u8)) -> Self {
        Self {
            color,
            ambient: 0.2,
            diffuse: 0.3,
            specular: 0.8,
            shininess: 128.0,
            reflectivity: 0.5,
            surface: SurfaceType::Metal,
            ..Default::default()
        }
    }

    pub fn glass() -> Self {
        Self {
            color: (255, 255, 255),
            ambient: 0.0,
            diffuse: 0.1,
            specular: 0.9,
            shininess: 256.0,
            reflectivity: 0.2,
            transparency: 0.9,
            refractive_index: 1.5,
            texture: TextureType::None,
            surface: SurfaceType::Glass,
        }
    }
}

/// Types of light sources
#[derive(Clone, Debug, PartialEq)]
pub enum LightSourceType {
    Point,
    Directional,
    Spot { cutoff_angle: f32, direction: Point3D },
}

/// A light source in the scene
#[derive(Clone, Debug, PartialEq)]
pub struct LightSource {
    pub position: Point3D,
    pub color: (u8, u8, u8),
    pub intensity: f32,
    pub light_type: LightSourceType,
}

impl LightSource {
    pub fn point(position: Point3D, color: (u8, u8, u8), intensity: f32) -> Self {
        Self {
            position,
            color,
            intensity,
            light_type: LightSourceType::Point,
        }
    }

    pub fn directional(direction: Point3D, color: (u8, u8, u8), intensity: f32) -> Self {
        // For directional lights, position is treated as the direction vector
        Self {
            position: direction.normalize(),
            color,
            intensity,
            light_type: LightSourceType::Directional,
        }
    }
}

/// Information about a ray-object intersection
#[derive(Clone, Debug)]
pub struct HitRecord {
    pub t: f32,
    pub point: Point3D,
    pub normal: Point3D,
    pub material: OpticalMaterial,
}
