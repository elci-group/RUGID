use crate::cell::CellId;
use crate::scheduler::{Scheduler, MegaCellId};
use crate::morphism::ShapeType;
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone, PartialEq)]
pub struct DeltaFrame {
    pub t_start: u64,
    pub t_end: u64,
    pub delta_r: f32,
    pub delta_g: f32,
    pub delta_b: f32,
    pub delta_a: f32,
    // Spatial deltas (relativistic/percent)
    pub delta_origin_p: f32,
    pub delta_origin_s: f32,
    pub delta_extent_p: f32,
    pub delta_extent_s: f32,
    // Morphism deltas
    /// Progress to advance in morphing animation (0.0 to 1.0 per tick)
    pub delta_morph_progress: f32,
    /// Target shape to morph into (if morphing)
    pub target_shape: Option<ShapeType>,
}

impl DeltaFrame {
    pub fn zero(t_start: u64, t_end: u64) -> Self {
        Self {
            t_start,
            t_end,
            delta_r: 0.0,
            delta_g: 0.0,
            delta_b: 0.0,
            delta_a: 0.0,
            delta_origin_p: 0.0,
            delta_origin_s: 0.0,
            delta_extent_p: 0.0,
            delta_extent_s: 0.0,
            delta_morph_progress: 0.0,
            target_shape: None,
        }
    }

    pub fn add(&mut self, other: &DeltaFrame) {
        self.delta_r += other.delta_r;
        self.delta_g += other.delta_g;
        self.delta_b += other.delta_b;
        self.delta_a += other.delta_a;
        self.delta_origin_p += other.delta_origin_p;
        self.delta_origin_s += other.delta_origin_s;
        self.delta_extent_p += other.delta_extent_p;
        self.delta_extent_s += other.delta_extent_s;
        self.delta_morph_progress += other.delta_morph_progress;
        // For target_shape, the last non-None value wins
        if other.target_shape.is_some() {
            self.target_shape = other.target_shape.clone();
        }
    }

    pub fn subtract(&mut self, other: &DeltaFrame) {
        self.delta_r -= other.delta_r;
        self.delta_g -= other.delta_g;
        self.delta_b -= other.delta_b;
        self.delta_a -= other.delta_a;
        self.delta_origin_p -= other.delta_origin_p;
        self.delta_origin_s -= other.delta_origin_s;
        self.delta_extent_p -= other.delta_extent_p;
        self.delta_extent_s -= other.delta_extent_s;
        self.delta_morph_progress -= other.delta_morph_progress;
        // Note: Subtracting shape deltas is tricky - for reversion, we clear the target
        if other.target_shape.is_some() {
            self.target_shape = None;
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum DeltaType {
    /// Persist in the cell state, modify baseline.
    /// Must declare an inverse for reversibility.
    Adhesive { inverse: Box<DeltaFrame> }, 
    /// Do not persist, leave no residue.
    Greasy,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DeltaProgram {
    pub target: CellId,
    pub schedule: Vec<DeltaFrame>,
    pub delta_type: DeltaType,
    pub accumulation: DeltaFrame, // Tracks total applied delta for Greasy reversion
}

impl DeltaProgram {
    pub fn new(target: CellId, schedule: Vec<DeltaFrame>, delta_type: DeltaType) -> Self {
        Self {
            target,
            schedule,
            delta_type,
            accumulation: DeltaFrame::zero(0, 0),
        }
    }
}

/// Parses movement syntax: `20,20<md<80,80~5`
/// Supports modifiers: `{~}` (yoyo/inverted cycle), `[-]` (simple cycle)
/// Input format: `[MODIFIERS]START_X,START_Y<md<END_X,END_Y~DURATION`
pub fn parse_movement(input: &str, start_time: u64) -> Result<Vec<DeltaFrame>, String> {
    let input = input.trim();
    
    // Check modifiers
    let is_yoyo = input.contains("{~}");
    let is_cycle = input.contains("[~]");
    
    // Clean input of modifiers
    let clean_input = input.replace("{~}", "").replace("[~]", "");
    
    // Parse parts: START<md<END~DURATION
    // Split by '~' first to get duration
    let parts: Vec<&str> = clean_input.split('~').collect();
    if parts.len() != 2 {
        return Err("Missing duration (~Ts)".to_string());
    }
    
    let duration_str = parts[1];
    let duration: u64 = duration_str.parse().map_err(|_| "Invalid duration")?;
    
    // Split remaining by '<md<'
    let coords_part = parts[0];
    let coords: Vec<&str> = coords_part.split("<md<").collect();
    if coords.len() != 2 {
        return Err("Invalid coordinate format (expected START<md<END)".to_string());
    }
    
    let start_coords: Vec<&str> = coords[0].split(',').collect();
    let end_coords: Vec<&str> = coords[1].split(',').collect();
    
    if start_coords.len() != 2 || end_coords.len() != 2 {
        return Err("Coordinates must be x,y".to_string());
    }
    
    let start_x: f32 = start_coords[0].parse().map_err(|_| "Invalid start x")?;
    let start_y: f32 = start_coords[1].parse().map_err(|_| "Invalid start y")?;
    let end_x: f32 = end_coords[0].parse().map_err(|_| "Invalid end x")?;
    let end_y: f32 = end_coords[1].parse().map_err(|_| "Invalid end y")?;
    
    // Convert percentages to 0.0-1.0 range if they are > 1.0?
    // User example: 20,20 ... 80,80. Usually implies 20% -> 0.2.
    // I'll assume input is 0-100 and convert to 0.0-1.0
    let s_x = start_x / 100.0;
    let s_y = start_y / 100.0;
    let e_x = end_x / 100.0;
    let e_y = end_y / 100.0;
    
    let delta_x = e_x - s_x;
    let delta_y = e_y - s_y;
    
    let mut frames = Vec::new();
    
    // Velocity per tick
    // If duration is in seconds, and we assume 1 tick = 1 second (or user meant ticks?)
    // User said "~Ts" implies seconds.
    // We need to know ticks per second. Assuming 1 tick = 1 unit of duration for now.
    // If duration is 0, panic or error.
    if duration == 0 {
        return Err("Duration cannot be 0".to_string());
    }
    
    let vx = delta_x / duration as f32;
    let vy = delta_y / duration as f32;
    
    // Create forward frame
    let mut frame = DeltaFrame::zero(start_time, start_time + duration);
    frame.delta_origin_p = vy; // Primary axis = Y (usually)
    frame.delta_origin_s = vx; // Secondary axis = X (usually)
    
    frames.push(frame);
    
    if is_yoyo {
        // Add return frame
        let mut return_frame = DeltaFrame::zero(start_time + duration, start_time + duration * 2);
        return_frame.delta_origin_p = -vy;
        return_frame.delta_origin_s = -vx;
        frames.push(return_frame);
    } else if is_cycle {
        // Simple cycle: [~]
        // This implies a "sawtooth" pattern: A -> B, then snap back to A.
        // Since we are using Greasy deltas by default in the demo, the "snap back" happens automatically
        // when the program ends and the accumulated delta is subtracted.
        // So for a single cycle, this is identical to the standard behavior.
        // However, to distinguish it, we could potentially add a "hold" or "repeat" if the engine supported it.
        // For this demo, we'll treat it as a standard forward pass, relying on Greasy for the snap-back.
        // If we wanted to emulate a "box" pattern more explicitly, we could add a 0-duration return frame?
        // No, Greasy handles the subtraction.
    }
    
    Ok(frames)
}

pub struct TemporalMemory {
    // Map of CellId -> List of active programs
    programs: HashMap<CellId, Vec<DeltaProgram>>,
    // Map of CellId -> MegaCellId (Spatial Location)
    cell_locations: HashMap<CellId, MegaCellId>,
    current_time: u64,
    scheduler: Scheduler,
}

impl TemporalMemory {
    pub fn new(max_megacells: usize) -> Self {
        Self {
            programs: HashMap::new(),
            cell_locations: HashMap::new(),
            current_time: 0,
            scheduler: Scheduler::new(max_megacells),
        }
    }

    pub fn add_program(&mut self, program: DeltaProgram) {
        self.programs.entry(program.target).or_default().push(program);
    }

    pub fn update_cell_location(&mut self, cell_id: CellId, mega_cell: MegaCellId) {
        self.cell_locations.insert(cell_id, mega_cell);
    }

    /// Advance time and return the NET deltas for each affected cell for this tick.
    /// Net Delta = (Sum of active Adhesive Δs) + (Sum of active Greasy Δs) - (Sum of finished Greasy accumulations)
    pub fn tick(&mut self) -> HashMap<CellId, DeltaFrame> {
        self.current_time += 1;
        let mut frame_outputs = HashMap::new();

        // 1. Identify active MegaCells
        let mut active_megacells = HashSet::new();
        
        // Iterate programs to find which cells are active
        for (cell_id, programs) in &self.programs {
             if !programs.is_empty() {
                 // Look up MegaCellId
                 if let Some(mc_id) = self.cell_locations.get(cell_id) {
                     active_megacells.insert(*mc_id);
                 } else {
                     // Default to 0 if unknown (fallback)
                     active_megacells.insert(MegaCellId(0));
                 }
             }
        }

        for mc in active_megacells {
            self.scheduler.schedule(mc);
        }

        // 2. Get batch
        let allowed_megacells: HashSet<MegaCellId> = self.scheduler.next_batch().into_iter().collect();

        // Iterate over all cells and their programs
        for (cell_id, programs) in self.programs.iter_mut() {
            let mc_id = self.cell_locations.get(cell_id).cloned().unwrap_or(MegaCellId(0));
            
            // If this cell's MegaCell is NOT in the allowed batch, skip it.
            if !allowed_megacells.contains(&mc_id) {
                continue;
            }

            let mut net_delta = DeltaFrame::zero(self.current_time, self.current_time + 1);
            let mut active_cell = false;
            
            let mut finished_indices = Vec::new();

            for (i, prog) in programs.iter_mut().enumerate() {
                // Find frame for current time
                let frame = prog.schedule.iter().find(|f| {
                    self.current_time >= f.t_start && self.current_time < f.t_end
                });

                if let Some(f) = frame {
                    net_delta.add(f);
                    
                    // Track accumulation for Greasy deltas
                    if let DeltaType::Greasy = prog.delta_type {
                        prog.accumulation.add(f);
                    }
                    active_cell = true;
                }

                // Check if program is finished
                let is_finished = !prog.schedule.iter().any(|f| f.t_end > self.current_time);
                if is_finished {
                    finished_indices.push(i);
                }
            }

            // Process finished programs (revert greasy)
            for i in finished_indices.into_iter().rev() {
                let prog = &programs[i];
                if let DeltaType::Greasy = prog.delta_type {
                    net_delta.subtract(&prog.accumulation);
                    active_cell = true;
                }
                programs.remove(i);
            }

            if active_cell {
                frame_outputs.insert(*cell_id, net_delta);
            }
        }

        frame_outputs
    }
    pub fn get_programs(&self, cell_id: CellId) -> Option<&Vec<DeltaProgram>> {
        self.programs.get(&cell_id)
    }

    pub fn get_active_cell_ids(&self) -> Vec<CellId> {
        self.programs.keys().cloned().collect()
    }

    pub fn current_time(&self) -> u64 {
        self.current_time
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cell::Cell;

    use crate::geometry::VectorRegion;

    #[test]
    fn test_greasy_reversion() {
        let mut memory = TemporalMemory::new(100);
        let cell = Cell::new(VectorRegion::new(0.0, 0.0, 1.0, 1.0));

        // Greasy program: +0.1 for 2 ticks
        let frame1 = DeltaFrame {
            t_start: 1,
            t_end: 3,
            delta_r: 0.1,
            delta_g: 0.0,
            delta_b: 0.0,
            delta_a: 0.0,
            delta_origin_p: 0.0,
            delta_origin_s: 0.0,
            delta_extent_p: 0.0,
            delta_extent_s: 0.0,
            delta_morph_progress: 0.0,
            target_shape: None,
        };
        
        let program = DeltaProgram::new(
            cell.id,
            vec![frame1],
            DeltaType::Greasy,
        );

        memory.add_program(program);

        // Tick 1: +0.1
        let out1 = memory.tick();
        assert_eq!(out1.get(&cell.id).unwrap().delta_r, 0.1);

        // Tick 2: +0.1
        let out2 = memory.tick();
        assert_eq!(out2.get(&cell.id).unwrap().delta_r, 0.1);

        // Tick 3: Program ends. Should revert total (+0.2) -> -0.2
        let out3 = memory.tick();
        // Floating point comparison
        let val = out3.get(&cell.id).unwrap().delta_r;
        assert!((val - -0.2).abs() < 1e-6);
    }

    #[test]
    fn test_adhesive_persistence() {
        let mut memory = TemporalMemory::new(100);
        let cell = Cell::new(VectorRegion::new(0.0, 0.0, 1.0, 1.0));

        // Adhesive program: +0.1 for 2 ticks
        let frame1 = DeltaFrame {
            t_start: 1,
            t_end: 3,
            delta_r: 0.1,
            delta_g: 0.0,
            delta_b: 0.0,
            delta_a: 0.0,
            delta_origin_p: 0.0,
            delta_origin_s: 0.0,
            delta_extent_p: 0.0,
            delta_extent_s: 0.0,
            delta_morph_progress: 0.0,
            target_shape: None,
        };
        
        // Mock inverse
        let inverse = Box::new(DeltaFrame::zero(0, 0));

        let program = DeltaProgram::new(
            cell.id,
            vec![frame1],
            DeltaType::Adhesive { inverse },
        );

        memory.add_program(program);

        // Tick 1: +0.1
        let out1 = memory.tick();
        assert_eq!(out1.get(&cell.id).unwrap().delta_r, 0.1);

        // Tick 2: +0.1
        let out2 = memory.tick();
        assert_eq!(out2.get(&cell.id).unwrap().delta_r, 0.1);

        // Tick 3: Program ends. Should NOT revert.
        let out3 = memory.tick();
        // Should be empty or zero if no other programs
        assert!(!out3.contains_key(&cell.id));
    }

    #[test]
    fn test_parse_movement() {
        let input = "20,20<md<80,80~5";
        let frames = parse_movement(input, 0).unwrap();
        
        assert_eq!(frames.len(), 1);
        let f = &frames[0];
        assert_eq!(f.t_start, 0);
        assert_eq!(f.t_end, 5);
        
        // 20->80 = 60% = 0.6
        // Duration 5
        // Velocity = 0.6 / 5 = 0.12
        assert!((f.delta_origin_p - 0.12).abs() < 1e-5); // Y
        assert!((f.delta_origin_s - 0.12).abs() < 1e-5); // X
    }

    #[test]
    fn test_parse_movement_yoyo() {
        let input = "{~}20,20<md<80,80~5";
        let frames = parse_movement(input, 0).unwrap();
        
        assert_eq!(frames.len(), 2);
        
        let f1 = &frames[0];
        assert_eq!(f1.t_start, 0);
        assert_eq!(f1.t_end, 5);
        assert!((f1.delta_origin_p - 0.12).abs() < 1e-5);
        
        let f2 = &frames[1];
        assert_eq!(f2.t_start, 5);
        assert_eq!(f2.t_end, 10);
        assert!((f2.delta_origin_p - -0.12).abs() < 1e-5);
    }
}
