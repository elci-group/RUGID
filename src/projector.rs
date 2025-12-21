use crate::signal::Signal;
use crate::diffuser::{Projection, Layer, BehaviourFlag};
use crate::temporal::{DeltaProgram, DeltaType};
use crate::cell::CellId;

/// A projector is a pure function that maps signals to cell-space deltas (programs).
///
/// Properties:
/// * Stateless
/// * Deterministic
/// * Side-effect free
pub trait Projector<T> {
    fn project(&self, signal: &Signal<T>) -> Projection;
}

pub struct EmptyProjector;

impl<T> Projector<T> for EmptyProjector {
    fn project(&self, _signal: &Signal<T>) -> Projection {
        // Return empty projection
        Projection {
            delta: DeltaProgram::new(CellId(0), vec![], DeltaType::Greasy),
            layer: Layer::Content,
            behaviour: BehaviourFlag::None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::signal::Signal;
    use crate::temporal::DeltaFrame;

    struct MockProjector;

    impl Projector<i32> for MockProjector {
        fn project(&self, signal: &Signal<i32>) -> Projection {
            // Mock program
            let program = DeltaProgram::new(
                CellId::next(),
                vec![DeltaFrame::zero(0, 10)],
                DeltaType::Greasy,
            );

            Projection {
                delta: program,
                layer: Layer::Content,
                behaviour: BehaviourFlag::None,
            }
        }
    }

    #[test]
    fn test_projector_execution() {
        let sig = Signal::new(42);
        let projector = MockProjector;
        let proj = projector.project(&sig);

        assert_eq!(proj.delta.schedule.len(), 1);
        assert_eq!(proj.layer, Layer::Content);
    }
}
