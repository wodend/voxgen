use crate::buffer::{Rgba, VolumeBuffer};
use std::path::Path;

use line_drawing::Bresenham;

pub enum Command {
    Draw,
    Move,
    Left,
    Right,
}

/// The drawing turtle.
#[derive(Debug)]
pub struct Turtle {
    x: u32,
    y: u32,
    heading: f32,
}

/// The canvas for the turtle to draw on.
///
/// - Use basic turtle graphics commands
/// - Intepret L System strings to generate models
/// - Save outputs as MagicaVoxel .vox files
/// 
/// # Examples
///
/// Draw a line and save the output.
/// ```
/// use voxgen::turtle::{Canvas, Command};
///
/// /// Default turtle state is (x=0, y=0, heading=0) where heading=0 means the
/// /// turtle is facing east.
/// let mut canvas = Canvas::new(3, 3, 3);
/// 
/// canvas.turtle(Command::Move, 1, 0.0);
/// canvas.turtle(Command::Left, 0, std::f32::consts::FRAC_PI_2);
/// canvas.turtle(Command::Draw, 2, 0.0);
/// canvas.save("test/volumes/mid_y_line.vox");
/// # Ok::<(), std::io::Error>(())
/// ```
///
pub struct Canvas {
    buf: VolumeBuffer<Rgba>,
    state: Turtle,
}

impl Canvas {
    pub fn new(size_x: u32, size_y: u32, size_z: u32) -> Canvas {
        Canvas {
            buf: VolumeBuffer::new(size_x, size_y, size_z),
            state: Turtle {
                x: 0,
                y: 0,
                heading: 0.0,
            },
        }
    }

    /// # Examples
    ///
    /// Draw a line and save the output.
    /// ```
    /// use voxgen::turtle::{Canvas, Command};
    ///
    /// /// Default turtle state is (x=0, y=0, heading=0) where heading=0 means the
    /// /// turtle is facing east.
    /// let mut canvas = Canvas::new(3, 3, 3);
    /// 
    /// /// Draw a line with step size 2 and angle increment of 0
    /// canvas.turtle(Command::Draw, 2, 0.0);
    /// canvas.save("test/volumes/line.vox");
    /// # Ok::<(), std::io::Error>(())
    /// ```
    ///
    pub fn turtle(&mut self, command: Command, step_size: u32, angle_increment: f32) {
        match command {
            Command::Draw => {
                let x_1 = self.state.x + (step_size * self.state.heading.cos() as u32);
                let y_1 = self.state.y + (step_size * self.state.heading.sin() as u32);
                println!("h {:?} sh {:?} ch {:?}", self.state.heading, self.state.heading.sin(), self.state.heading.cos());
                for (x, y) in Bresenham::new((self.state.x as i32, self.state.y as i32), (x_1 as i32, y_1 as i32)) {
                    println!("{:?}", (x, y));
                    *self.buf.get_mut(x as u32, y as u32, 0) = Rgba([0, 0, 0, 255]);
                }
                self.state.x = x_1;
                self.state.y = y_1;
            },
            Command::Move => {
                let x_1 = self.state.x + (step_size * self.state.heading.cos() as u32);
                let y_1 = self.state.y + (step_size * self.state.heading.sin() as u32);
                println!("{:?} {:?}", (self.state.x, self.state.y), (x_1, y_1));
                self.state.x = x_1;
                self.state.y = y_1;
            },
            Command::Left => {
                println!("{:?} - {:?}", self.state, angle_increment);
                self.state.heading -= angle_increment;
                println!("{:?}", self.state);
                if self.state.heading < 0.0 {
                    self.state.heading = std::f32::consts::TAU - self.state.heading;
                }
                println!("{:?}", self.state);
            },
            Command::Right => {
                self.state.heading += angle_increment;
                if self.state.heading > std::f32::consts::TAU {
                    self.state.heading = 0.0 + self.state.heading;
                }
                self.state.heading = self.state.heading.clamp(0.0, std::f32::consts::TAU);
                println!("{:?}", self.state);
            },
        }
    }

    pub fn save<P>(&self, path: P) -> std::io::Result<()>
    where
        P: AsRef<Path>,
    {
        self.buf.save(path)?;
        Ok(())
    }
}
