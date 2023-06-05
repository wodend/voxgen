// Render an order 8 dragon curve with a rainbow gradient.
use voxgen::l_system::{LSystem, RenderOptions};

fn main() {
    // Define an L System
    let l_system = LSystem::new(
        "dragon",
        "L",
        vec![
            "L→L+R+",
            "R→-L-R",
        ]
    );
    // Render the L System as a MagicaVoxel .vox file.
    RenderOptions::new()
        .derivation_length(8)
        .offset_x(10.0)
        .offset_y(-15.0)
        .rainbow(true)
        .render(l_system);
}