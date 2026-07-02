use crate::config::MapImageConfig;
use crate::geometry::{CellMap, ComputesCellMap, Geometry};
use ::image::DynamicImage;
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::ops::{Add, Mul, Sub};
use std::path::PathBuf;

pub mod config;
pub mod geometry;

mod document;
mod image_handling;

fn _generate_map(args: &[String]) -> Result<(), Box<dyn Error>> {
    let _parsed_config = config::parse_config(args);
    unimplemented!()
}

fn _load_map(path: String) -> DynamicImage {
    image::open(path).unwrap()
}

pub fn compute_cell_map(map_image_config: &MapImageConfig, geometry: Geometry) -> CellMap {
    let map_margin = map_image_config
        .image_margins
        .unwrap_or(PixelPoint { x: 0, y: 0 });
    geometry.compute_cell_map(
        temp_get_image_dimensions(&map_image_config.image_file),
        map_margin,
    )
}

fn temp_get_image_dimensions(_image_path: &PathBuf) -> PixelPoint {
    unimplemented!()
}

#[derive(Deserialize, Serialize, PartialEq, Debug, Clone, Copy)]
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

impl Sub<PixelPoint> for PixelPoint {
    type Output = PixelPoint;

    fn sub(self, rhs: PixelPoint) -> PixelPoint {
        PixelPoint {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        }
    }
}

impl Sub<PositionDelta> for PixelPoint {
    type Output = PixelPoint;

    fn sub(self, rhs: PositionDelta) -> PixelPoint {
        PixelPoint {
            x: self.x - rhs.x as i32,
            y: self.y - rhs.y as i32,
        }
    }
}

impl Mul<i32> for PixelPoint {
    type Output = PixelPoint;

    fn mul(self, rhs: i32) -> PixelPoint {
        PixelPoint {
            x: self.x * rhs,
            y: self.y * rhs,
        }
    }
}

impl Mul<f32> for PixelPoint {
    type Output = PixelPoint;

    fn mul(self, rhs: f32) -> PixelPoint {
        PixelPoint {
            x: (self.x as f32 * rhs) as i32,
            y: (self.y as f32 * rhs) as i32,
        }
    }
}

/// A Box in pixel space, defined by two opposite corners.
///
/// Remember that the top-left corner of an image has X,Y coordinate (0,0).
#[derive(Debug, PartialEq, Deserialize, Serialize)]
pub struct PixelBox {
    pub top_left_corner: PixelPoint,
    pub bottom_right_corner: PixelPoint,
}

impl Default for PixelBox{
    fn default() -> Self {
        PixelBox{ top_left_corner: PixelPoint {x: 0, y:0}, bottom_right_corner: PixelPoint {x:0, y:0} }
    }
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
