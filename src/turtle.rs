use crate::buffer::{Rgba, VolumeBuffer};
use std::path::Path;

use line_drawing::Bresenham;

/// The drawing turtle.
#[derive(Copy, Clone, Debug)]
pub struct Turtle {
    x: i32,
    y: i32,
    heading: f32,
}

/// Draw a `VolumeBuffer` using LOGO-style turtle graphics commands.
/// 
/// - Use basic turtle graphics commands
/// - Save outputs as magicavoxel .vox files
/// 
/// # Examples
///
/// Draw a line and save the output.
/// ```
/// use voxgen::turtle::TurtleGraphics;
///
/// let mut turtle = TurtleGraphics::new(3, 3, 3);
/// 
/// /// Move the turtle 1 step forward (east) without drawing.
/// turtle.step(1.0);
/// 
/// /// Turn the turtle pi/2 radians left (facing north).
/// turtle.left(std::f32::consts::FRAC_PI_2);
/// 
/// /// Draw a line 2 steps down the middle of the y axis.
/// turtle.draw(2.0);
/// 
/// /// Save the current drawing as a magicavoxel .vox file.
/// turtle.buf().save("test/volumes/mid_y_line.vox").unwrap();
/// ```
pub struct TurtleGraphics {
    buf: VolumeBuffer<Rgba>,
    state: Turtle,
}

impl TurtleGraphics {
    /// Create a new `TurtleGraphics` object of the given dimensions.
    /// 
    /// The `VolumeBuffer` is initially empty, and the turtle is at position (0,
    /// 0, 0) with a heading of 0.0 radians (facing east).
    pub fn new(size_x: u32, size_y: u32, size_z: u32) -> TurtleGraphics {
        TurtleGraphics {
            buf: VolumeBuffer::new(size_x, size_y, size_z),
            state: Turtle {
                x: 0,
                y: 0,
                heading: 0.0,
            },
        }
    }

    /// Move the turtle without drawing a line.
    pub fn step(&mut self, step_size: f32) {
        self.state.x = self.state.x + (step_size * self.state.heading.cos()) as i32;
        self.state.y = self.state.y + (step_size * self.state.heading.sin()) as i32;
    }

    /// Move the turtle and draw a line along it's path.
    /// 
    /// The turtle moves `step_size` voxels in the direction of it's current
    /// `heading`.
    pub fn draw(&mut self, step_size: f32) {
        let (x0, y0) = (self.state.x, self.state.y);
        self.step(step_size);
        let (x1, y1) = (self.state.x, self.state.y);
        for (x, y) in Bresenham::new((x0, y0), (x1, y1)) {
            *self.buf.get_mut(x as u32, y as u32, 0) = Rgba([0, 0, 0, 255]);
        }
    }

    /// Rotate the turtle `angle_increment` radians to the left.
    pub fn right(&mut self, angle_increment: f32) {
        self.state.heading -= angle_increment;
    }

    /// Rotate the turtle `angle_increment` radians to the right.
    pub fn left(&mut self, angle_increment: f32) {
        self.state.heading += angle_increment;
    }

    /// Get the current state of the turtle.
    pub fn state(&mut self) -> Turtle {
        self.state
    }

    /// Get the current state of the turtle.
    pub fn buf(&mut self) -> &VolumeBuffer<Rgba> {
        &self.buf
    }
}
