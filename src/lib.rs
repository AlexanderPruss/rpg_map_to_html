use crate::config::MapImageConfig;
use crate::geometry::{CellMap, ComputesCellMap};
use serde::Deserialize;
use std::error::Error;
use std::ops::{Add, Mul};

pub mod config;
pub mod geometry;

fn generate_map(args: &[String]) -> Result<(), Box<dyn Error>> {
    let parsed_config = config::parse_config(args);
    unimplemented!()
}

pub fn compute_cell_map(map_image_config: &MapImageConfig) -> CellMap {
    let map_margin = map_image_config
        .image_margins
        .unwrap_or(PixelPoint { x: 0, y: 0 });
    map_image_config.geometry.compute_cell_map(
        temp_get_image_dimensions(&map_image_config.image_file),
        map_margin,
    )
}

fn temp_get_image_dimensions(image_path: &String) -> PixelPoint {
    unimplemented!()
}

#[derive(Deserialize, PartialEq, Debug, Clone, Copy)]
pub struct PixelPoint {
    pub x: i32,
    pub y: i32,
}

impl Add<PixelPoint> for PixelPoint {
    type Output = PixelPoint;

    fn add(self, rhs: PixelPoint) -> PixelPoint {
        PixelPoint {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

/// A Box in pixel space, defined by two opposite corners.
///
/// Remember that the top-left corner of an image has X,Y coordinate (0,0).
pub struct PixelBox {
    pub top_left_corner: PixelPoint,
    pub bottom_right_corner: PixelPoint,
}

#[derive(Debug, Clone, Copy)]
struct PositionDelta {
    x: f32,
    y: f32,
}

impl Add<PositionDelta> for PositionDelta {
    type Output = Self;

    fn add(self, rhs: PositionDelta) -> Self {
        PositionDelta {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

impl Mul<u8> for PositionDelta {
    type Output = Self;

    fn mul(self, rhs: u8) -> Self {
        PositionDelta {
            x: self.x * rhs as f32,
            y: self.y * rhs as f32,
        }
    }
}

impl From<PositionDelta> for PixelPoint {
    fn from(value: PositionDelta) -> Self {
        PixelPoint {
            x: value.x.round() as i32,
            y: value.y.round() as i32,
        }
    }
}
