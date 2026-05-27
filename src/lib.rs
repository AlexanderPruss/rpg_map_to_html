use config::Config;
use std::error::Error;
use serde::Deserialize;
use crate::config::MapImageConfig;
use crate::geometry::{CellMap, ComputesCellMap};

mod config;
mod geometry;

#[derive(Deserialize, PartialEq, Debug, Clone, Copy)]
struct PixelPoint {
    x: i32,
    y: i32,
}

/// A Box in pixel space, defined by two opposite corners.
///
/// Remember that the top-left corner of an image has X,Y coordinate (0,0).
struct PixelBox {
    top_left_corner: PixelPoint,
    bottom_right_corner: PixelPoint,
}

fn generate_map(args: &[String]) -> Result<(), Box<dyn Error>> {
    let parsed_config = config::parse_config(args);
    unimplemented!()
}

fn compute_map_geometry(map_image_config: &MapImageConfig) -> CellMap {
    todo!();
}