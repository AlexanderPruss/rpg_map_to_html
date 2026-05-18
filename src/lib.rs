use std::error::Error;
use config::Config;
mod config;
mod geometry;

struct PixelPoint {
    x: i32,
    y: i32
}


/// A Box in pixel space, defined by two opposite corners.
///
/// Remember that the top-left corner of an image has X,Y coordinate (0,0).
struct PixelBox {
    top_left_corner: PixelPoint,
    bottom_right_corner: PixelPoint
}

fn generate_map(args: &[String]) -> Result<(), Box<dyn Error>>{
    let parsed_config = config::parse_config(args);
    unimplemented!()
}