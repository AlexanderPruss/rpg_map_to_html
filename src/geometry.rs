use crate::{PixelBox, PixelPoint};
use serde::Deserialize;
use std::collections::HashMap;

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

pub struct CellMap {
    cells_by_coordinate: HashMap<String, Cell>,
}

pub struct Cell {
    coordinate: String,
    neighbor_coordinates: Vec<String>,

    center_point: PixelPoint,
    bounding_polygon: BoundingPolygon,
}

/// A list of points defining a polygon in pixel space.
#[derive(Clone, Debug, PartialEq)]
pub struct BoundingPolygon {
    points: Vec<PixelPoint>,
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
                .map(|point|
                    PixelPoint {
                    x: point.x.clamp(0, max.x),
                    y: point.y.clamp(0, max.y),
                })
                .collect(),
        }
    }
}

pub trait ComputesCellMap {
    fn compute_cell_map<'map>(
        &self,
        map_dimensions: PixelPoint,
        map_margin: PixelPoint,
    ) -> CellMap;
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
    //TODO: These serialization tests need to be integration tests with assertions

    #[test]
    #[ignore]
    fn should_deserialize_complete_hex_geometry<'map>() {
        let serialized = r#"
            {
                "type": "Hexagons",
                "flat_sides": "FlatVerticalSides",
                "number_of_rows": 2,
                "number_of_columns": 3,
                "hexagon_height": 40.2,
                "hexagon_width": 50.1,
                "row_details": "UNIFORM",
                "column_details": "UNIFORM",
                "filled_top_left_corner": "EMPTY"
            }
        "#;
        let _hex_geometry: Geometry = serde_json::from_str(&serialized).unwrap();
    }

    #[test]
    #[ignore]
    fn should_deserialize_hex_geometry_without_optional_fields() {
        let serialized = r#"
            {
                "type": "Hexagons",
                "flat_sides": "FlatVerticalSides",
                "number_of_rows": 2,
                "number_of_columns": 3,
                "hexagon_height": 40.2,
                "hexagon_width": 50.1,
                "filled_top_left_corner": "EMPTY"
            }
        "#;
        let _hex_geometry: Geometry = serde_json::from_str(&serialized).unwrap();
    }
}
