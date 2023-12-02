# graph-plot

A simple graph plotting tool with a couple of graph theory algorithms, 
implemented using the [Bevy](https://bevyengine.org/) engine.

## Building
Requires the [Rust](https://www.rust-lang.org/) toolchain for building.

Build using `cargo build`

The current configuration in `Cargo.toml` is set up specifically for a linux-wayland platform. 
Removing the feature flags for bevy and reenabling `default-features` might allow you to run 
it on other platforms.

## Features
- Create and move nodes and edges
- Directed or undirected edges
- Adjacency Matrix + eigenvectors + eigenvalues
- Spanning tree visualization and Djikstra's shortest path visualization
- Bipartite graph visualization
- Node physics
- Labels!
- Colors!
- Wow!
