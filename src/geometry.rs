use crate::{PixelBox, PixelPoint};
use serde::Deserialize;
use std::collections::{HashMap, HashSet};

pub mod hexagons;

/// Describes the structure of the RPG map. The map's [Geometry] is used to identify map cells and neighbors.
#[derive(Deserialize, Debug)]
#[serde(tag = "type")]
pub enum Geometry {
    /// A hex map.
    ///
    /// Currently one where all rows have the same number of hexes, and all columns have the same number of hexes
    Hexagons {
        definition: hexagons::HexagonGeometryDefinition,
    },
}

#[derive(Debug, PartialEq)]
pub struct CellMap {
    cells_by_coordinate: HashMap<String, Cell>,
}

#[derive(Debug, PartialEq)]
pub struct Cell {
    coordinate: String,
    neighbor_coordinates: HashSet<String>,

    center_point: PixelPoint,
    bounding_polygon: BoundingPolygon,
}

/// A list of points defining a polygon in pixel space.
#[derive(Clone, Debug)]
pub struct BoundingPolygon {
    points: Vec<PixelPoint>,
}

///The starting point and direction of the polygon is irrelevant for equality.
/// Polygons are equal if the points result in the same polygon.
impl PartialEq for BoundingPolygon {
    fn eq(&self, other: &Self) -> bool {
        if self.points.len() != other.points.iter().len() {
            return false
        }
        if self.points.len() == 0 {
            return false
        }
        let first_point = self.points.first().unwrap();
        let index_of_first_point_in_other = other.points.iter().position(|point| *point == *first_point);
        let mut matching_index = match index_of_first_point_in_other {
            None => return false,
            Some(index) => index
        };
        let mut match_in_same_direction = true;
        let backward_matching_index = matching_index;
        for index in 0..self.points.len() {
            if matching_index == self.points.iter().len() {
                matching_index = 0
            }
            if self.points[index] != other.points[matching_index] {
                match_in_same_direction = false;
                break;
            }
            matching_index+=1;
        }
        if match_in_same_direction {
            return true
        }
        let mut match_in_reverse_direction = true;
        let mut matching_index = backward_matching_index;
        for index in 0..self.points.len() {
            if self.points[index] != other.points[matching_index] {
                match_in_reverse_direction = false;
                break;
            }
            matching_index = match matching_index {
                0 => self.points.len() - 1,
                _ => matching_index - 1
            };
        }
        match_in_reverse_direction
    }

    fn ne(&self, other: &Self) -> bool {
        !self.eq(other)
    }
}

impl BoundingPolygon {
    pub fn restrict_to_bounding_box(&self, bounding_box: PixelBox) -> BoundingPolygon {
        BoundingPolygon {
            points: self
                .points
                .iter()
                .map(|point| PixelPoint {
                    x: point.x.clamp(
                        bounding_box.top_left_corner.x,
                        bounding_box.bottom_right_corner.x,
                    ),
                    y: point.y.clamp(
                        bounding_box.top_left_corner.y,
                        bounding_box.bottom_right_corner.y,
                    ),
                })
                .collect(),
        }
    }

    pub fn offset_by(&self, offset: PixelPoint) -> BoundingPolygon {
        BoundingPolygon {
            points: self
                .points
                .iter()
                .map(|point| PixelPoint {
                    x: point.x + offset.x,
                    y: point.y + offset.y,
                })
                .collect(),
        }
    }

    pub fn clamp(&self, max: PixelPoint) -> BoundingPolygon {
        BoundingPolygon {
            points: self
                .points
                .iter()
                .map(|point| PixelPoint {
                    x: point.x.clamp(0, max.x),
                    y: point.y.clamp(0, max.y),
                })
                .collect(),
        }
    }
}

pub trait ComputesCellMap {
    fn compute_cell_map<'map>(&self, map_dimensions: PixelPoint, map_margin: PixelPoint)
    -> CellMap;
}

impl ComputesCellMap for Geometry {
    fn compute_cell_map(&self, map_dimensions: PixelPoint, map_margin: PixelPoint) -> CellMap {
        match self {
            Geometry::Hexagons {
                definition: hex_geometry_defn,
            } => hex_geometry_defn.compute_cell_map(map_dimensions, map_margin),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    mod bounding_polygon {

        mod restrict_to_bounding_box {
            #[test]
            fn clamps_all_polygon_points() {
                unimplemented!()
            }
        }

        mod offset_by {
            #[test]
            fn offsets_the_polygon_in_pixel_space() {
                unimplemented!()
            }
        }

        mod clamp {
            #[test]
            fn clamps_with_an_implicit_lower_bound(){
                unimplemented!()
            }
        }

        mod equality {

            #[test]
            fn equals_polygons_with_identical_point_lists(){
                unimplemented!()
            }

            #[test]
            fn equals_polygons_with_identical_points_but_a_different_starting_point(){
                unimplemented!()
            }

            #[test]
            fn does_not_equal_polygons_with_a_different_size(){
                unimplemented!()
            }

            #[test]
            fn does_not_equal_polygons_with_different_points(){
                unimplemented!()
            }

            #[test]
            fn does_not_equal_polygons_with_identical_points_in_a_different_iteration_order(){
                unimplemented!()
            }

            #[test]
            fn does_not_panic_when_either_list_is_empty(){
                unimplemented!()
            }
        }
    }

}
