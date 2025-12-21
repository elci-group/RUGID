use crate::geometry::VectorRegion;
use crate::cell::CellId;
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LayoutDirection {
    StackPrimary,
    StackSecondary,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LayoutConstraint {
    Fixed(f32), // Absolute units (0.0 - 1.0 range usually)
    Flex(f32),  // Weight
}

pub struct LayoutNode {
    pub direction: LayoutDirection,
    pub constraint: LayoutConstraint,
    pub children: Vec<LayoutNode>,
    pub cell_id: Option<CellId>,
}

impl LayoutNode {
    pub fn leaf(cell_id: CellId, constraint: LayoutConstraint) -> Self {
        Self {
            direction: LayoutDirection::StackPrimary, // Irrelevant for leaf
            constraint,
            children: Vec::new(),
            cell_id: Some(cell_id),
        }
    }

    pub fn container(direction: LayoutDirection, constraint: LayoutConstraint, children: Vec<LayoutNode>) -> Self {
        Self {
            direction,
            constraint,
            children,
            cell_id: None,
        }
    }
}

pub struct LayoutSolver;

impl LayoutSolver {
    pub fn solve(root: &LayoutNode, available_space: VectorRegion) -> HashMap<CellId, VectorRegion> {
        let mut results = HashMap::new();
        Self::solve_recursive(root, available_space, &mut results);
        results
    }

    fn solve_recursive(node: &LayoutNode, region: VectorRegion, results: &mut HashMap<CellId, VectorRegion>) {
        if let Some(cell_id) = node.cell_id {
            results.insert(cell_id, region);
            return;
        }

        if node.children.is_empty() {
            return;
        }

        // Calculate total fixed size and total flex weight along the stacking axis
        let mut total_fixed = 0.0;
        let mut total_flex = 0.0;

        for child in &node.children {
            match child.constraint {
                LayoutConstraint::Fixed(size) => total_fixed += size,
                LayoutConstraint::Flex(weight) => total_flex += weight,
            }
        }

        // Determine available space for flex items
        let axis_size = match node.direction {
            LayoutDirection::StackPrimary => region.extent_p,
            LayoutDirection::StackSecondary => region.extent_s,
        };

        let remaining_space = (axis_size - total_fixed).max(0.0);
        
        let mut current_pos = match node.direction {
            LayoutDirection::StackPrimary => region.origin_p,
            LayoutDirection::StackSecondary => region.origin_s,
        };

        for child in &node.children {
            let child_size = match child.constraint {
                LayoutConstraint::Fixed(size) => size,
                LayoutConstraint::Flex(weight) => {
                    if total_flex > 0.0 {
                        (weight / total_flex) * remaining_space
                    } else {
                        0.0
                    }
                }
            };

            let child_region = match node.direction {
                LayoutDirection::StackPrimary => VectorRegion {
                    origin_p: current_pos,
                    origin_s: region.origin_s,
                    extent_p: child_size,
                    extent_s: region.extent_s,
                },
                LayoutDirection::StackSecondary => VectorRegion {
                    origin_p: region.origin_p,
                    origin_s: current_pos,
                    extent_p: region.extent_p,
                    extent_s: child_size,
                },
            };

            Self::solve_recursive(child, child_region, results);
            current_pos += child_size;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cell::Cell;

    #[test]
    fn test_layout_solver() {
        let cell1 = Cell::new(VectorRegion::new(0.0, 0.0, 0.0, 0.0));
        let cell2 = Cell::new(VectorRegion::new(0.0, 0.0, 0.0, 0.0));

        // Root: StackPrimary (Vertical/Flow)
        // Child 1: Fixed 0.2
        // Child 2: Flex 1.0
        let root = LayoutNode::container(
            LayoutDirection::StackPrimary,
            LayoutConstraint::Fixed(1.0), // Root fills parent
            vec![
                LayoutNode::leaf(cell1.id, LayoutConstraint::Fixed(0.2)),
                LayoutNode::leaf(cell2.id, LayoutConstraint::Flex(1.0)),
            ]
        );

        let space = VectorRegion::new(0.0, 0.0, 1.0, 1.0);
        let layout = LayoutSolver::solve(&root, space);

        let r1 = layout.get(&cell1.id).unwrap();
        let r2 = layout.get(&cell2.id).unwrap();

        // Check Cell 1 (Fixed 0.2)
        assert!((r1.extent_p - 0.2).abs() < 1e-5);
        assert!((r1.origin_p - 0.0).abs() < 1e-5);
        
        // Check Cell 2 (Flex 1.0 -> Takes remaining 0.8)
        assert!((r2.extent_p - 0.8).abs() < 1e-5);
        assert!((r2.origin_p - 0.2).abs() < 1e-5);
    }
}
