//! IronBeam Blender Converter
//! 
//! Handles conversion between Blender files and RUGID formats.

use std::path::{Path, PathBuf};
use std::process::Command;
use crate::ironbeam::formats::rgd::RgdDocument;

#[derive(Debug)]
pub enum ConvertError {
    Io(std::io::Error),
    BlenderFailed(Option<i32>),
    ParseError(String),
}

impl From<std::io::Error> for ConvertError {
    fn from(err: std::io::Error) -> Self {
        ConvertError::Io(err)
    }
}

pub struct BlenderConverter {
    /// Path to Blender executable
    pub blender_path: PathBuf,
    
    /// Temporary directory for intermediate files
    pub temp_dir: PathBuf,
}

impl BlenderConverter {
    pub fn new(blender_path: PathBuf, temp_dir: PathBuf) -> Self {
        Self { blender_path, temp_dir }
    }

    /// Import a .blend file into RUGID
    pub fn import(&self, blend_path: &Path) -> Result<RgdDocument, ConvertError> {
        // 1. Export from Blender to glTF using Python script
        let gltf_path = self.temp_dir.join("export.gltf");
        self.run_blender_script(blend_path, &gltf_path, EXPORT_SCRIPT)?;
        
        // 2. Parse glTF (Placeholder)
        // let gltf = GltfParser::parse(&gltf_path)?;
        
        // 3. Convert to RGD (Placeholder)
        // let rgd = self.gltf_to_rgd(gltf)?;
        
        // 4. Cleanup
        if gltf_path.exists() {
            std::fs::remove_file(&gltf_path)?;
        }
        
        Ok(RgdDocument::new())
    }
    
    fn run_blender_script(&self, input: &Path, output: &Path, script: &str) -> Result<(), ConvertError> {
        let script_path = self.temp_dir.join("script.py");
        std::fs::write(&script_path, script)?;
        
        let status = Command::new(&self.blender_path)
            .args(&[
                "--background",
                input.to_str().unwrap(),
                "--python", script_path.to_str().unwrap(),
                "--", output.to_str().unwrap(),
            ])
            .status()?;
        
        if !status.success() {
            return Err(ConvertError::BlenderFailed(status.code()));
        }
        
        Ok(())
    }
}

const EXPORT_SCRIPT: &str = r#"
import bpy
import sys

output_path = sys.argv[sys.argv.index("--") + 1]
bpy.ops.export_scene.gltf(filepath=output_path, export_format='GLTF_SEPARATE')
"#;

const IMPORT_SCRIPT: &str = r#"
import bpy
import sys

input_path = sys.argv[sys.argv.index("--") + 1]
output_path = sys.argv[sys.argv.index("--") + 2]

bpy.ops.wm.read_factory_settings(use_empty=True)
bpy.ops.import_scene.gltf(filepath=input_path)
bpy.ops.wm.save_as_mainfile(filepath=output_path)
"#;
