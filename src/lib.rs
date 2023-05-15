//! Procedural MagicaVoxel .vox generation.
//!
//! Provides utilities for generating
//! [MagicaVoxel](https://ephtracy.github.io/) models using popular 3D
//! procedural generation techniques.

/// A voxel grid data structure.
///
/// Implemented based on the [image](https://crates.io/crates/image) crate.
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
