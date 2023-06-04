//! Procedural MagicaVoxel .vox generation.
//!
//! Provides utilities for generating
//! [MagicaVoxel](https://ephtracy.github.io/) models using popular 3D
//! procedural generation techniques.

#![doc = include_str!("../README.md")]

/// A voxel grid data structure.
///
/// Implemented based on the [image](https://crates.io/crates/image) crate.
///
/// # Examples
///
/// Draw a simple 2D red cross and save as a MagicaVoxel .vox file.
/// ```
/// use voxgen::buffer::{ArrayVoxelBuffer, Rgba, VoxelBuffer};
///
/// let mut vol = ArrayVoxelBuffer::new(32, 32, 32);
///
/// for x in 15..=17 {
///     for y in 8..24 {
///         *vol.voxel_mut(x, y, 0) = Rgba([255, 0, 0, 255]);
///         *vol.voxel_mut(y, x, 0) = Rgba([255, 0, 0, 255]);
///     }
/// }
///
/// vol.save("test/volumes/red_cross.vox")?;
/// # Ok::<(), std::io::Error>(())
/// ```
pub mod buffer;

/// Draw on voxel buffers using turtle graphics.
///
/// Use basic turtle graphics commands and save outputs as magicavoxel .vox
/// files.  Implemented based on the descriptions in [The Algorithmic Beauty of
/// Plants](http://algorithmicbotany.org/papers/abop/abop-ch1.pdf).
///
/// # Examples
///
/// Draw a line and save the output.
/// ```
/// use voxgen::turtle::TurtleGraphics;
///
/// let mut turtle = TurtleGraphics::new(3, 3, 3);
///
/// // Move the turtle 1 step forward (east) without drawing.
/// turtle.step(1.0);
///
/// // Turn the turtle pi/2 radians left (facing north).
/// turtle.left(std::f32::consts::FRAC_PI_2);
///
/// // Draw a line 2 steps down the middle of the y axis.
/// turtle.draw(2.0);
///
/// // Save the current drawing as a magicavoxel .vox file.
/// turtle.buf().save("test/volumes/mid_y_line.vox").unwrap();
/// ```
///
/// Draw a gradient line.
/// ```
/// # use voxgen::turtle::TurtleGraphics;
/// use enterpolation::{
///     Curve,
///     linear::ConstEquidistantLinear,
/// };
/// use palette::{LinSrgba, Srgba};
///
/// // A gradient of evenly spaced rainbow colors.
/// let grad1 = ConstEquidistantLinear::<f32, _, 7>::equidistant_unchecked([
///     LinSrgba::new(1.0, 0.0, 0.0, 1.0),
///     LinSrgba::new(1.0, 1.0, 0.0, 1.0),
///     LinSrgba::new(0.0, 1.0, 0.0, 1.0),
///     LinSrgba::new(0.0, 1.0, 1.0, 1.0),
///     LinSrgba::new(0.0, 0.0, 1.0, 1.0),
///     LinSrgba::new(1.0, 0.0, 1.0, 1.0),
///     LinSrgba::new(1.0, 0.0, 0.0, 1.0),
/// ]);
///
/// let mut turtle = TurtleGraphics::new(8, 32, 3);
/// let mut g: Vec<[u8; 4]> = Vec::new();
///
/// let step_size = 31.0;
/// for (i, c1) in grad1
///     .take(step_size as usize + 1)
///     .enumerate()
/// {
///     let c1 = Srgba::from_linear(c1).into();
///     g.push(c1);
/// }
///
/// turtle.step(8.0 / 2.0);
/// turtle.left(std::f32::consts::FRAC_PI_2);
/// turtle.draw_gradient(step_size, &g);
/// turtle.buf().save("test/volumes/gradient_line.vox").unwrap();
/// ```
pub mod turtle;

/// Inteprets L System strings and draws them using turtle graphics.
///
/// Implemented based on the descriptions in [The Algorithmic Beauty of
/// Plants](http://algorithmicbotany.org/papers/abop/abop-ch1.pdf).
///
/// # Examples
///
/// Render a Koch curve.
/// ```
/// use voxgen::l_system::{LSystem, RenderOptions};
///
///
/// let l_system = LSystem::new(
///     "koch",
///     "F-F-F-F",
///     vec!["F→F-F+F+FF-F-F+F"],
/// );
/// // Builder pattern for custom rendering options.
/// // Default path is test/volumes/{l_system_name}_{derivation_length}.vox.
/// RenderOptions::new()
///     .offset_x(-20.0)
///     .offset_y(-20.0)
///     .render(l_system);
/// ```
///
/// Render a dragon curve.
/// ```
/// # use voxgen::l_system::{LSystem, RenderOptions};
/// let l_system = LSystem::new(
///     "dragon",
///     "L",
///     vec![
///         "L→L+R+",
///         "R→-L-R",
///     ]
/// );
/// RenderOptions::new()
///     .derivation_length(8)
///     .offset_x(10.0)
///     .offset_y(-15.0)
///     .rainbow(true)
///     .render(l_system);
/// ```
///
/// Render a Sierpinski gasket.
/// ```
/// # use voxgen::l_system::{LSystem, RenderOptions};
/// let l_system = LSystem::new(
///     "sierpinski-gasket",
///     "R",
///     vec![
///         "L→R+L+R",
///         "R→L-R-L",
///     ]
/// );
/// RenderOptions::new()
///     .derivation_length(3)
///     .step_size(4.0)
///     .angle_increment(std::f32::consts::FRAC_PI_3)
///     .offset_y(-20.0)
///     .render(l_system);
/// ```
///
/// Render a Hilbert curve.
/// ```
/// # use voxgen::l_system::{LSystem, RenderOptions};
/// let l_system = LSystem::new(
///     "hilbert",
///     "A",
///     vec![
///         "A→+BF-AFA-FB+",
///         "B→-AF+BFB+FA-",
///     ],
/// );
/// RenderOptions::new()
///     .size_x(127)
///     .size_y(127)
///     .offset_x(63.0)
///     .offset_y(-63.0)
///     .derivation_length(6)
///     .render(l_system);
/// ```
pub mod l_system;
