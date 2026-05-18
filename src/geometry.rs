use std::collections::HashMap;
use serde::{Deserialize};
use crate::{PixelBox, PixelPoint};

pub mod hexagons;

#[derive(Deserialize, Debug)]
#[serde(tag = "type")]
pub enum Geometry {
    /// A hex map.
    ///
    /// Currently one where all rows have the same number of hexes, and all columns have the same number of hexes
    Hexagons(hexagons::HexagonGeometryDefinition)
}

pub struct CellMap<'map> {
    cells_by_coordinate: HashMap<&'map str, Cell>,
    neighbors_by_coordinate: HashMap<&'map str, Vec<&'map str>>
}

pub struct Cell {
    coordinate: String,

    center_point: PixelPoint,
    /// A list of points defining a polygon that draws this cell in pixel space. This can be approximate.
    bounding_polygon: Vec<PixelPoint>
}

impl Cell {
    pub fn restrict_bounding_polygon_to_bounding_box(&self, bounding_box: PixelBox) -> Vec<PixelPoint> {
        self.bounding_polygon.iter().map(|point| {
            let mut x = point.x;
            let mut y = point.y;
            if x > bounding_box.bottom_right_corner.x {
                x = bounding_box.bottom_right_corner.x;
            }
            if y > bounding_box.bottom_right_corner.y {
                y = bounding_box.bottom_right_corner.y;
            }
            if x < bounding_box.top_left_corner.x {
                x = bounding_box.top_left_corner.x
            }
            if y < bounding_box.top_left_corner.y {
                y = bounding_box.top_left_corner.y
            }
            PixelPoint{x, y}
        }).collect()
    }
}

trait ComputesCellMap {
    fn compute_cell_map<'map>(&self) -> CellMap<'map>;
}

impl ComputesCellMap for Geometry {
    fn compute_cell_map<'map>(&self) -> CellMap<'map> {
        match self {
            Geometry::Hexagons(hex_geometry_defn) => {
                hex_geometry_defn.compute_cell_map()
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    //TODO: These tests need to move to the hexagon module and get assertions

    #[test]
    fn should_deserialize_complete_hex_geometry<'map>() {
/*        let mut mappo = CellMap{
            cells_by_coordinate: HashMap::new()
        };
        mappo.cells_by_coordinate.insert("computeCellMap", Cell{coordinate: String::new()});*/

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