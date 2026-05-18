use serde::{Deserialize};
use crate::geometry::Geometry;

#[derive(Deserialize, Debug)]
pub struct Config {
    pub target_map_file : String,
    pub image_directory : String,
    pub mode: GenerationMode,
    pub geometry: Geometry
}

#[derive(Deserialize, Debug)]
pub enum GenerationMode {
    /// (Re)creates a map, discarding the previous one if it already exists
    CreateOverwrite,
    /// Updates the images of an existing map, adding new pages as needed
    Update
}

pub fn parse_config(args: &[String]) -> Config {
    unimplemented!()
}