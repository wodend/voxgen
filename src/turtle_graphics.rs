use line_drawing::Bresenham;

use crate::voxel_buffer::{ArrayVoxelBuffer, Rgba, VoxelBuffer};

/// The drawing turtle.
#[derive(Copy, Clone, Debug)]
pub struct Turtle {
    x: i32,
    y: i32,
    heading: f32,
    color: Rgba,
}

/// Draw an `ArrayVoxelBuffer` using LOGO-style turtle graphics commands.
pub struct TurtleGraphics {
    buf: ArrayVoxelBuffer<Rgba>,
    state: Turtle,
}

impl TurtleGraphics {
    /// Create a new `TurtleGraphics` object of the given dimensions.
    ///
    /// The `ArrayVoxelBuffer` is initially empty. The turtle starts at position
    /// `(0, 0, 0)` with a heading of `0.0` radians (facing east) with RGBA
    /// drawing color `[0, 0, 0, 255]`.
    pub fn new(size_x: u32, size_y: u32, size_z: u32) -> TurtleGraphics {
        TurtleGraphics {
            buf: ArrayVoxelBuffer::new(size_x, size_y, size_z),
            state: Turtle {
                x: 0,
                y: 0,
                heading: 0.0,
                color: Rgba([0, 0, 0, 255])
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
            *self.buf.voxel_mut(x as u32, y as u32, 0) = self.state.color;
        }
    }

    /// Set the turtle drawing color to the RGBA value of `color`.
    pub fn color(&mut self, color: Rgba) {
        self.state.color = color;
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
    pub fn buf(&mut self) -> &ArrayVoxelBuffer<Rgba> {
        &self.buf
    }
}
