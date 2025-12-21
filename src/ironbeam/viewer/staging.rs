//! IronBeam Staging System
//! 
//! Manages the collection of 3D models present in the scene.

use crate::cell::CellId;
use crate::cad::CADMesh;
use crate::geometry3d::{Point3D, Rotation3D};
use std::collections::{HashMap, HashSet};

#[derive(Clone, Debug)]
pub struct Transform3D {
    pub position: Point3D,
    pub rotation: Rotation3D,
    pub scale: Point3D,
}

impl Default for Transform3D {
    fn default() -> Self {
        Self {
            position: Point3D::zero(),
            rotation: Rotation3D::zero(),
            scale: Point3D::new(1.0, 1.0, 1.0),
        }
    }
}

impl Transform3D {
    pub fn translation(pos: Point3D) -> Self {
        Self {
            position: pos,
            ..Default::default()
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct MaterialOverride {
    pub color: Option<(u8, u8, u8)>,
    pub opacity: Option<f32>,
    pub wireframe: bool,
}

#[derive(Clone, Debug)]
pub struct StagedModel {
    pub id: CellId,
    pub mesh: CADMesh,
    pub transform: Transform3D,
    pub material: MaterialOverride,
    pub visibility: bool,
    pub locked: bool,
    pub layer: u8,
}

pub enum StagingAction {
    Add(StagedModel),
    Remove(CellId),
    Transform { id: CellId, before: Transform3D, after: Transform3D },
    Group(Vec<CellId>),
    Ungroup(CellId),
}

pub struct StagingManager {
    pub models: HashMap<CellId, StagedModel>,
    pub active_selection: HashSet<CellId>,
    pub clipboard: Vec<StagedModel>,
    pub undo_stack: Vec<StagingAction>,
    pub redo_stack: Vec<StagingAction>,
}

impl StagingManager {
    pub fn new() -> Self {
        Self {
            models: HashMap::new(),
            active_selection: HashSet::new(),
            clipboard: Vec::new(),
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
        }
    }

    pub fn add_model(&mut self, mesh: CADMesh, position: Point3D) -> CellId {
        let id = CellId::generate();
        let model = StagedModel {
            id,
            mesh,
            transform: Transform3D::translation(position),
            material: MaterialOverride::default(),
            visibility: true,
            locked: false,
            layer: 0,
        };
        self.models.insert(id, model.clone());
        self.undo_stack.push(StagingAction::Add(model));
        self.redo_stack.clear();
        id
    }
    
    pub fn select(&mut self, id: CellId, additive: bool) {
        if !additive {
            self.active_selection.clear();
        }
        self.active_selection.insert(id);
    }
    
    pub fn deselect_all(&mut self) {
        self.active_selection.clear();
    }
    
    pub fn get_selection_center(&self) -> Option<Point3D> {
        if self.active_selection.is_empty() {
            return None;
        }
        
        let mut sum = Point3D::zero();
        let mut count = 0.0;
        
        for id in &self.active_selection {
            if let Some(model) = self.models.get(id) {
                sum += model.transform.position;
                count += 1.0;
            }
        }
        
        if count > 0.0 {
            Some(Point3D::new(sum.x / count, sum.y / count, sum.z / count))
        } else {
            None
        }
    }
}
