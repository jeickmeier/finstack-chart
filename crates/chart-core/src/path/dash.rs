//! Destination-specific dashed stroke lowering over the shared bounded flattener.
use super::PathGeometry;
use crate::{ChartResult, scene::PathCommand};

impl PathGeometry {
    /// Flatten a curve and split its stroke into dash segments at destination precision.
    /// The even, positive pattern uses destination units and resets at every MoveTo.
    /// Fill geometry and semantic anchors are not changed. Both intermediate vertices
    /// and final commands are bounded independently by `max_commands`.
    pub fn dashed(
        &self,
        dashes: &[f64],
        max_error: f64,
        max_commands: usize,
    ) -> ChartResult<Vec<PathCommand>> {
        // Reject malformed patterns before geometric work, including empty geometry.
        crate::scene::dash_polyline(&[], dashes, max_commands)?;
        let flat = self.flatten(max_error, max_commands)?;
        let mut commands = Vec::new();
        for sub in flat.subpaths {
            for (index, point) in sub.points.into_iter().enumerate() {
                super::require(
                    commands.len() < max_commands,
                    "Dashed path input limit exceeded",
                )?;
                commands.push(if index == 0 {
                    PathCommand::MoveTo(point)
                } else {
                    PathCommand::LineTo(point)
                });
            }
            if sub.closed {
                super::require(
                    commands.len() < max_commands,
                    "Dashed path input limit exceeded",
                )?;
                commands.push(PathCommand::Close);
            }
        }
        crate::scene::dash_polyline(&commands, dashes, max_commands)
    }
}
