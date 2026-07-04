use crate::config::{Config};
use crate::geometry::{ ComputesCellMap};
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::io;
use std::ops::{Add, Mul, Sub};
use std::path::{ PathBuf};

pub mod config;
pub mod geometry;

pub mod document;
pub mod image_handling;

pub static LAST_USED_CONFIG : &str = "last_used_config.json";

/// A convenience method - loads the config and does all the logic for you. Computes a geometry,
/// creates all the images, creates/updates the markdown content and final html document.
///
/// Also saves a visualization of the geometry and the cell-map used.
pub fn generate_docs(config: Config)  {
    //Config
    let image_path = &config.map_image.image_file;
    let map_image = image::open(image_path).unwrap();
    let map_dimensions = PixelPoint{x: map_image.width() as i32, y: map_image.height() as i32 };
    let map_margin = config.map_image.image_margins.unwrap_or(PixelPoint { x: 0, y: 0 });
    let target_directory = &config.target_directory;

    //Geometry
    let cell_map = config.geometry.compute_cell_map(
        map_dimensions,
        map_margin,
    );

    //Image handling
    let (skip_empty_cells, image_handling) = image_handling::image_config::resolve_config(&config.map_image.skip_empty_cells,&config.image_handling_config);
    let table_of_contents_images = image_handling::table_of_contents::save_table_of_contents_map_images(
        target_directory,
        &map_image,
        &image_handling.max_table_of_contents_map_image_size,
        &cell_map
    );
    image_handling::map_cutout::save_cutout_images(
        target_directory,
        &cell_map,
        &map_image,
        map_margin,
        &skip_empty_cells,
        &image_handling
    );

    //Document
    document::html::write_html_doc(target_directory, &config.title, &cell_map, image_handling.zoomed_in_map_image_size, table_of_contents_images, &config.template);

    //Cleanup
    image_handling::visualize_cell_map::save_cell_map_visualization(
        target_directory, &cell_map, &map_image, &image_handling.cell_outline_color
    );
    geometry::persist_cell_map_as_geometry(target_directory, cell_map);
    let config_path = PathBuf::from(LAST_USED_CONFIG);
    config::persist_config(&config, &config_path);
}

pub fn read_input_with_default(input: &mut String, default: String) -> String {
    input.clear();
    io::stdin().read_line(input).unwrap();
    match input.to_lowercase().trim() {
        "" => default,
        _ => input.trim().to_string()
    }
}

pub fn read_input(input: &mut String) -> String {
    input.clear();
    io::stdin().read_line(input).unwrap();
    input.trim().to_string()
}

pub fn read_input_yes_no_with_default(input: &mut String, default: bool) -> bool {
    loop {
        io::stdin().read_line(input).unwrap();
        match input.to_lowercase().trim() {
            "y" => return true,
            "" => return default,
            "n" => {
                return false
            }
            _ => {
                println!(
                    "Couldn't understand the input '{input}'. Please input Y or N."
                )
            }
        }
    }
}

pub fn read_input_until_valid_option(input: &mut String, options: Vec<&str>, default: &str) -> String{
    loop {
        let user_input = read_input_with_default(input, "Hexagons".to_string());
        match options.iter().find(|option| option.to_lowercase() == user_input.to_lowercase()){
            None => {println!("Did not understand the input '{input}'. Please input one of [{}] (default: {default}).", options.join(", "))}
            Some(option) => return option.to_string()
        }
    }
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
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct PixelBox {
    pub top_left_corner: PixelPoint,
    pub bottom_right_corner: PixelPoint,
}

impl Default for PixelBox {
    fn default() -> Self {
        PixelBox {
            top_left_corner: PixelPoint { x: 0, y: 0 },
            bottom_right_corner: PixelPoint { x: 0, y: 0 },
        }
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
