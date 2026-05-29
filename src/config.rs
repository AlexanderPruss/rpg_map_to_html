use crate::PixelPoint;
use crate::geometry::Geometry;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct Config {
    //TODO: Figure out what else we need at some point
    /// New files - images and html - will be saved in a child directory of [target_directory].
    pub target_directory: String,
    pub map_image: MapImageConfig,
}

#[derive(Deserialize, Debug)]
pub struct MapImageConfig {
    /// The file containing the map. ex: sweetestMap.png
    pub image_file: String,
    /// The contentless margin surrounding the map. Hexographer's default export has no margin.
    pub image_margins: Option<PixelPoint>,
    pub geometry: Geometry,
}

#[derive(Deserialize, Debug)]
pub enum GenerationMode {
    /// (Re)creates a map, discarding the previous one if it already exists
    CreateOverwrite,
    /// Updates the images of an existing map, adding new pages as needed
    Update,
}

pub fn parse_config(args: &[String]) -> Config {
    unimplemented!()
}
