use crate::temporal::DeltaProgram;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Layer {
    Background = 0,
    Content = 10,
    Overlay = 20,
    Debug = 100,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BehaviourFlag {
    /// Geometry locked at initial resolution
    Init,
    /// Applies directional translation vector
    Rail,
    /// Applies proportional scaling
    Dyna,
    /// Multiplies opacity contribution
    Fade(f32),
    /// No special behaviour
    None,
}

/// Wrapper for a projection result with metadata
#[derive(Debug, Clone)]
pub struct Projection {
    pub delta: DeltaProgram,
    pub layer: Layer,
    pub behaviour: BehaviourFlag,
}

pub struct Diffuser;

impl Diffuser {
    /// Resolves multiple projections into a single DeltaProgram (or list of them).
    /// For now, we assume all projections target the same cell (or we group them).
    /// Returns a list of resolved programs (one per target cell if mixed, or merged).
    pub fn resolve(projections: Vec<Projection>) -> Vec<DeltaProgram> {
        // In a real implementation, we would group by target cell first.
        // Here we assume single target or handle independently.
        
        let mut resolved_programs = Vec::new();

        for proj in projections {
            let mut program = proj.delta;
            
            // Apply behaviour flags to the schedule
            match proj.behaviour {
                BehaviourFlag::Fade(opacity) => {
                    for frame in &mut program.schedule {
                        frame.delta_a *= opacity;
                    }
                }
                _ => {} // Other flags might affect transform or geometry
            }

            resolved_programs.push(program);
        }

        // Note: We are NOT summing them here yet. 
        // The directive says "The final cell value is Σ(Projectors)".
        // But TemporalMemory sums them per tick.
        // So Diffuser just prepares/transforms them.
        
        resolved_programs
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cell::CellId;
    use crate::temporal::{DeltaType, DeltaFrame};

    #[test]
    fn test_diffuser_behaviour_application() {
        let cell_id = CellId::next();
        let mut frame = DeltaFrame::zero(0, 10);
        frame.delta_a = 1.0;

        let program = DeltaProgram::new(
            cell_id,
            vec![frame],
            DeltaType::Greasy,
        );

        let p = Projection {
            delta: program,
            layer: Layer::Content,
            behaviour: BehaviourFlag::Fade(0.5),
        };

        let resolved = Diffuser::resolve(vec![p]);
        assert_eq!(resolved.len(), 1);
        assert_eq!(resolved[0].schedule[0].delta_a, 0.5);
    }
}
