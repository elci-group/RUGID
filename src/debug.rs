use crate::temporal::TemporalMemory;
use crate::cell::CellId;

pub struct TemporalDebugger;

impl TemporalDebugger {
    pub fn inspect(memory: &TemporalMemory, cell_id: CellId) -> String {
        let mut report = String::new();
        report.push_str(&format!("Inspection for Cell {:?}:\n", cell_id));

        if let Some(programs) = memory.get_programs(cell_id) {
            report.push_str(&format!("  Active Programs: {}\n", programs.len()));
            for (i, prog) in programs.iter().enumerate() {
                report.push_str(&format!("    Program #{}: Type={:?}, Frames={}\n", 
                    i, prog.delta_type, prog.schedule.len()));
                report.push_str(&format!("      Accumulation: R={}, G={}, B={}, A={}\n",
                    prog.accumulation.delta_r,
                    prog.accumulation.delta_g,
                    prog.accumulation.delta_b,
                    prog.accumulation.delta_a
                ));
            }
        } else {
            report.push_str("  No active programs.\n");
        }

        report
    }

    pub fn dump_active_cells(memory: &TemporalMemory) -> Vec<CellId> {
        memory.get_active_cell_ids()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::temporal::{DeltaProgram, DeltaFrame, DeltaType};
    use crate::cell::Cell;
    use crate::geometry::VectorRegion;

    #[test]
    fn test_debugger_inspection() {
        let mut memory = TemporalMemory::new(100);
        let cell = Cell::new(VectorRegion::new(0.0, 0.0, 1.0, 1.0));

        let frame = DeltaFrame::zero(0, 10);
        let program = DeltaProgram::new(
            cell.id,
            vec![frame],
            DeltaType::Greasy,
        );

        memory.add_program(program);

        let report = TemporalDebugger::inspect(&memory, cell.id);
        assert!(report.contains("Active Programs: 1"));
        assert!(report.contains("Type=Greasy"));
    }
}
