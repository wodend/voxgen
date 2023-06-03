//! Procedural MagicaVoxel .vox generation.
//!
//! Provides utilities for generating
//! [MagicaVoxel](https://ephtracy.github.io/) models using popular 3D
//! procedural generation techniques.

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

/// Draws models using turtle graphics.
///
/// Implemented based on the descriptions in [The Algorithmic Beauty of
/// Plants](http://algorithmicbotany.org/papers/abop/abop-ch1.pdf).
pub mod turtle;

/// Inteprets L System strings and draws them using turtle graphics.
///
/// Implemented based on the descriptions in [The Algorithmic Beauty of
/// Plants](http://algorithmicbotany.org/papers/abop/abop-ch1.pdf).
pub mod l_system;
