use crate::geometry::Geometry::Hexagons;
use crate::geometry::hexagons::FilledTopLeftCorner::{EMPTY, FILLED};
use crate::geometry::hexagons::FlatSides::FlatVerticalSides;
use crate::geometry::hexagons::standardized::StandardizedHexCellMap;
use crate::geometry::hexagons::transform::InvertibleStandardizedGeometry;
use crate::geometry::{BoundingPolygon, Cell, CellMap, ComputesCellMap, Geometry};
use crate::{PixelBox, PixelPoint, read_input, read_input_until_valid_option};
use FlatSides::FlatHorizontalSides;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

mod transform;

mod standardized;

/// A hex map. All rows have the same number of hexes, all columns have the same number of hexes.
#[derive(Deserialize, Serialize, PartialEq, Debug, Clone)]
pub struct HexagonGeometryDefinition {
    pub flat_sides: FlatSides,
    pub filled_top_left_corner: FilledTopLeftCorner,
    pub number_of_rows: u8,
    pub number_of_columns: u8,
    /// The units here can be pixels, cm, whatever, so long as they're consistent with [hexagon_width](HexagonGeometryDefinition::hexagon_width)
    pub hexagon_height: f32,
    /// The units here can be pixels, cm, whatever, so long as they're consistent with [hexagon_height](HexagonGeometryDefinition::hexagon_height)
    pub hexagon_width: f32,
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy, PartialOrd, Ord)]
struct HexCellCoordinate {
    row: u8,
    column: u8,
}
impl HexCellCoordinate {
    ///This representation is identical to the one used in Hexographer/Worldographer.
    fn to_coordinate_string(&self) -> String {
        format!(
            "{}.{}",
            Self::pad_coordinate(self.column),
            Self::pad_coordinate(self.row)
        )
    }

    /// Pads a number so that it is at least three digits long.
    fn pad_coordinate(coordinate: u8) -> String {
        let prefix = if coordinate < 10 {
            "00"
        } else if coordinate < 100 {
            "0"
        } else {
            ""
        };
        format!("{}{}", prefix, coordinate)
    }
}

#[derive(Debug, PartialEq, Clone)]
struct HexCell {
    hex_coordinate: HexCellCoordinate,
    neighbor_coordinates: HashSet<HexCellCoordinate>,
    center_point: PixelPoint,
    bounding_polygon: BoundingPolygon,
}

impl HexCell {
    /// The inscribed rectangle for a hex is just the rectangle made by the four points closest to
    /// the center. Since it's made up of points of the HexCell's bounding polygon, we just need
    /// to see which four points make a square.
    ///
    /// Written during a 40c heatwave.
    fn compute_inscribed_rectangle(&self) -> PixelBox {
        let mut square_points: Vec<PixelPoint> = vec![];
        let mut points_iter = self.bounding_polygon.points.iter();
        let mut previous_point = points_iter.next().unwrap();
        let mut index = 1;
        for point in points_iter {
            if previous_point.x == point.x || previous_point.y == point.y {
                square_points.push(*previous_point);
                square_points.push(*point);
                square_points.push(*self.bounding_polygon.points.get((index + 2) % 6).unwrap());
                square_points.push(*self.bounding_polygon.points.get((index + 3) % 6).unwrap());
                break;
            }
            previous_point = point;
            index += 1;
        }

        let mut square_points_iter = square_points.into_iter();
        let first_square_point = square_points_iter.next().unwrap();
        let mut top_left_corner = first_square_point;
        let mut bottom_right_corner = first_square_point;
        for square_point in square_points_iter {
            if square_point.x < top_left_corner.x {
                top_left_corner.x = square_point.x
            }
            if square_point.y < top_left_corner.y {
                top_left_corner.y = square_point.y
            }
            if square_point.x > bottom_right_corner.x {
                bottom_right_corner.x = square_point.x
            }
            if square_point.y > bottom_right_corner.y {
                bottom_right_corner.y = square_point.y
            }
        }
        PixelBox {
            top_left_corner,
            bottom_right_corner,
        }
    }
}

/// Whether the vertical or horizontal sides of the hexagon are flat.
#[derive(Deserialize, Serialize, Debug, PartialEq, Clone, Copy)]
pub enum FlatSides {
    FlatVerticalSides,
    FlatHorizontalSides,
}

/// Whether the top left corner of the map is filled in by a hex.
#[derive(Deserialize, Serialize, Debug, PartialEq, Clone, Copy)]
pub enum FilledTopLeftCorner {
    /// There's an empty hex at the top left of the map. For flat top hexes, the corner looks like this:
    ///
    ///``` picture
    ///                         ••••••••••
    ///                        •          •
    ///                       •            •
    ///             ••••••••••     (0,1)    •
    ///            •          •            •
    ///           •            •          •
    ///          •     (0,0)    ••••••••••
    ///           •            •
    ///            •          •
    ///             ••••••••••
    ///```
    EMPTY,
    /// There's no empty space at the top left of the map. For flat top hexes, the corner looks like this:
    ///
    ///``` picture
    ///             ••••••••••
    ///            •          •
    ///           •            •
    ///          •     (0,0)    ••••••••••
    ///           •            •          •
    ///            •          •            •
    ///             ••••••••••    (0,1)     •
    ///                       •            •
    ///                        •          •
    ///                         ••••••••••
    /// ```
    FILLED,
}

impl FilledTopLeftCorner {
    fn switch(&self) -> FilledTopLeftCorner {
        match self {
            EMPTY => FILLED,
            FILLED => EMPTY,
        }
    }
}

impl FlatSides {
    fn switch(&self) -> FlatSides {
        match self {
            FlatHorizontalSides => FlatVerticalSides,
            FlatVerticalSides => FlatHorizontalSides,
        }
    }
}

type HexCellMap = HashMap<HexCellCoordinate, HexCell>;

fn offset_map(hex_cell_map: HexCellMap, offset: PixelPoint) -> HexCellMap {
    if (offset == PixelPoint { x: 0, y: 0 }) {
        return hex_cell_map;
    }
    hex_cell_map
        .into_iter()
        .map(|(coordinate, cell)| {
            let cell_shifted_by_margin = HexCell {
                hex_coordinate: cell.hex_coordinate,
                neighbor_coordinates: cell.neighbor_coordinates,
                center_point: cell.center_point + offset,
                bounding_polygon: cell.bounding_polygon.offset_by(offset),
            };
            (coordinate, cell_shifted_by_margin)
        })
        .collect()
}
fn to_cell_map(hex_cell_map: HexCellMap) -> CellMap {
    CellMap {
        cells_by_coordinate: hex_cell_map
            .into_iter()
            .map(|(hex_coordinate, hex_cell)| {
                (
                    hex_coordinate.to_coordinate_string(),
                    Cell {
                        coordinate: hex_cell.hex_coordinate.to_coordinate_string(),
                        inscribed_rectangle: hex_cell.compute_inscribed_rectangle(),
                        neighbor_coordinates: hex_cell
                            .neighbor_coordinates
                            .into_iter()
                            .map(|hex_coordinate| hex_coordinate.to_coordinate_string())
                            .collect(),
                        center_point: hex_cell.center_point,
                        bounding_polygon: hex_cell.bounding_polygon,
                    },
                )
            })
            .collect(),
    }
}

impl ComputesCellMap for HexagonGeometryDefinition {
    fn compute_cell_map(&self, map_dimensions: PixelPoint, map_margin: PixelPoint) -> CellMap {
        let map_dimensions_without_margin = PixelPoint {
            x: map_dimensions.x - 2 * map_margin.x,
            y: map_dimensions.y - 2 * map_margin.y,
        };

        let invertible_standardized_geometry =
            InvertibleStandardizedGeometry::standardize(&self, map_dimensions_without_margin);
        let standardized_hex_cell_map: StandardizedHexCellMap =
            invertible_standardized_geometry.compute_standardized_cell_map();

        let hex_cell_map = standardized_hex_cell_map.invert_standardization();
        let hex_cell_map = offset_map(hex_cell_map, map_margin);
        to_cell_map(hex_cell_map)
    }
}

/// Creates a Hexagon Geometry by prompting the user with a terminal wizard.
pub fn generate_geometry_interactively() -> Geometry {
    let mut input = String::new();
    println!(
        "Does the hex map have flat vertical sides or flat horizontal sides? Vertical/Horizontal (default: Horizontal)"
    );
    let flat_sides = match read_input_until_valid_option(
        &mut input,
        vec!["vertical", "horizontal"],
        "horizontal",
    )
    .as_str()
    {
        "vertical" => FlatVerticalSides,
        _ => FlatHorizontalSides,
    };

    println!(
        r"
Filled:

   ..........
  .          .
 .            .
.     (0,0)    ..........
 .            .          .
  .          .            .
   ..........    (0,1)     .
             .            .
              .          .
               ..........

Empty:
               ..........
              .          .
             .            .
   ..........     (0,1)    .
  .          .            .
 .            .          .
.     (0,0)    ..........
 .            .
  .          .
   ..........

Is the top-left corner of the hex map filled or empty? Filled/Empty (default: Filled)
    "
    );
    let filled_top_left_corner =
        match read_input_until_valid_option(&mut input, vec!["filled", "empty"], "filled").as_str()
        {
            "filled" => FILLED,
            _ => EMPTY,
        };

    println!("Number of rows: ");
    let number_of_rows: u8 = read_input(&mut input).parse().unwrap();
    println!("Number of columns: ");
    let number_of_columns: u8 = read_input(&mut input).parse().unwrap();
    println!("Hexagon height (float, any units): (ex: 10.0)");
    let hexagon_height: f32 = read_input(&mut input).parse().unwrap();
    println!("Hexagon width (float, any units): (ex: 20.0)");
    let hexagon_width: f32 = read_input(&mut input).parse().unwrap();
    Hexagons {
        definition: HexagonGeometryDefinition {
            flat_sides,
            filled_top_left_corner,
            number_of_rows,
            number_of_columns,
            hexagon_height,
            hexagon_width,
        },
    }
}

#[cfg(test)]
pub(crate) mod fixtures;

#[cfg(test)]
mod test {

    mod deserialization {
        use super::super::*;
        use crate::geometry::Geometry;

        #[test]
        fn should_deserialize_complete_hex_geometry<'map>() {
            let serialized = r#"
            {
                "type": "Hexagons",
                "definition": {
                    "flat_sides": "FlatVerticalSides",
                    "number_of_rows": 2,
                    "number_of_columns": 3,
                    "hexagon_height": 40.2,
                    "hexagon_width": 50.1,
                    "filled_top_left_corner": "EMPTY"
                }
            }
        "#;
            let hex_geometry: Geometry = serde_json::from_str(&serialized).unwrap();
            match hex_geometry {
                Geometry::Hexagons { definition } => {
                    assert_eq!(
                        definition,
                        HexagonGeometryDefinition {
                            flat_sides: FlatVerticalSides,
                            number_of_rows: 2,
                            number_of_columns: 3,
                            hexagon_height: 40.2,
                            hexagon_width: 50.1,
                            filled_top_left_corner: EMPTY
                        }
                    )
                }
                Geometry::Generic { .. } => {
                    panic!("Should not have deserialized a Generic geometry.")
                }
            }
        }
    }

    mod hex_cell_coordinate {
        use super::super::*;

        #[test]
        fn stringifies_short_coordinates() {
            let origin = HexCellCoordinate { row: 0, column: 0 };
            let some_hex_coordinate = HexCellCoordinate { row: 3, column: 5 };

            assert_eq!("000.000", origin.to_coordinate_string());
            assert_eq!("005.003", some_hex_coordinate.to_coordinate_string());
        }

        #[test]
        fn stringifies_two_digit_coordinates() {
            let medium_coordinate = HexCellCoordinate {
                row: 11,
                column: 22,
            };
            assert_eq!("022.011", medium_coordinate.to_coordinate_string());
        }

        #[test]
        fn stringifies_three_digit_coordinates() {
            let long_coordinate = HexCellCoordinate {
                row: 111,
                column: 222,
            };
            assert_eq!("222.111", long_coordinate.to_coordinate_string());
        }
    }

    mod hex_cell_inscribed_rectangle {
        use crate::geometry::BoundingPolygon;
        use crate::geometry::hexagons::{HexCell, HexCellCoordinate};
        use crate::{PixelBox, PixelPoint};
        use std::collections::HashSet;

        #[test]
        fn identifies_rectangles_for_flat_bottom_hexes() {
            let mut hex_cell = HexCell {
                hex_coordinate: HexCellCoordinate { row: 0, column: 0 },
                neighbor_coordinates: HashSet::from([HexCellCoordinate { row: 9, column: 10 }]),
                center_point: PixelPoint { x: 15, y: 20 },
                bounding_polygon: BoundingPolygon {
                    points: vec![
                        PixelPoint { x: 10, y: 10 },
                        PixelPoint { x: 20, y: 10 },
                        PixelPoint { x: 30, y: 20 },
                        PixelPoint { x: 20, y: 30 },
                        PixelPoint { x: 10, y: 30 },
                        PixelPoint { x: 00, y: 20 },
                    ],
                },
            };
            let expected_rectangle = PixelBox {
                top_left_corner: PixelPoint { x: 10, y: 10 },
                bottom_right_corner: PixelPoint { x: 20, y: 30 },
            };
            for _ in 1..=6 {
                hex_cell.bounding_polygon.points.rotate_right(1);
                assert_eq!(expected_rectangle, hex_cell.compute_inscribed_rectangle());
            }
            hex_cell.bounding_polygon.points.reverse();
            for _ in 1..=6 {
                hex_cell.bounding_polygon.points.rotate_right(1);
                assert_eq!(expected_rectangle, hex_cell.compute_inscribed_rectangle());
            }
        }

        #[test]
        fn identifies_rectangles_for_flat_side_hexes() {
            let mut hex_cell = HexCell {
                hex_coordinate: HexCellCoordinate { row: 0, column: 0 },
                neighbor_coordinates: HashSet::from([HexCellCoordinate { row: 9, column: 10 }]),
                center_point: PixelPoint { x: 15, y: 20 },
                bounding_polygon: BoundingPolygon {
                    points: vec![
                        PixelPoint { x: 10, y: 10 },
                        PixelPoint { x: 20, y: 20 },
                        PixelPoint { x: 20, y: 30 },
                        PixelPoint { x: 10, y: 40 },
                        PixelPoint { x: 00, y: 30 },
                        PixelPoint { x: 00, y: 20 },
                    ],
                },
            };
            let expected_rectangle = PixelBox {
                top_left_corner: PixelPoint { x: 00, y: 20 },
                bottom_right_corner: PixelPoint { x: 20, y: 30 },
            };
            for _ in 1..=6 {
                hex_cell.bounding_polygon.points.rotate_right(1);
                assert_eq!(expected_rectangle, hex_cell.compute_inscribed_rectangle());
            }
            hex_cell.bounding_polygon.points.reverse();
            for _ in 1..=6 {
                hex_cell.bounding_polygon.points.rotate_right(1);
                assert_eq!(expected_rectangle, hex_cell.compute_inscribed_rectangle());
            }
        }
    }

    mod offset_map {
        use super::super::*;

        #[test]
        fn offsets_all_pixels_of_the_cell_map() {
            let offset = PixelPoint { x: 10, y: 100 };
            let cells: Vec<HexCell> = Vec::from([
                HexCell {
                    hex_coordinate: HexCellCoordinate { row: 0, column: 0 },
                    neighbor_coordinates: HashSet::new(),
                    center_point: PixelPoint { x: 1, y: 2 },
                    bounding_polygon: BoundingPolygon {
                        points: vec![PixelPoint { x: 3, y: 4 }],
                    },
                },
                HexCell {
                    hex_coordinate: HexCellCoordinate { row: 1, column: 2 },
                    neighbor_coordinates: HashSet::new(),
                    center_point: PixelPoint { x: 5, y: 6 },
                    bounding_polygon: BoundingPolygon {
                        points: vec![PixelPoint { x: 7, y: 8 }],
                    },
                },
            ]);
            let expected_cells: Vec<HexCell> = Vec::from([
                HexCell {
                    hex_coordinate: HexCellCoordinate { row: 0, column: 0 },
                    neighbor_coordinates: HashSet::new(),
                    center_point: PixelPoint { x: 11, y: 102 },
                    bounding_polygon: BoundingPolygon {
                        points: vec![PixelPoint { x: 13, y: 104 }],
                    },
                },
                HexCell {
                    hex_coordinate: HexCellCoordinate { row: 1, column: 2 },
                    neighbor_coordinates: HashSet::new(),
                    center_point: PixelPoint { x: 15, y: 106 },
                    bounding_polygon: BoundingPolygon {
                        points: vec![PixelPoint { x: 17, y: 108 }],
                    },
                },
            ]);
            let hex_cell_map: HexCellMap = cells
                .into_iter()
                .map(|cell| (cell.hex_coordinate, cell))
                .collect();
            let expected_hex_cell_map: HexCellMap = expected_cells
                .into_iter()
                .map(|cell| (cell.hex_coordinate, cell))
                .collect();

            assert_eq!(expected_hex_cell_map, offset_map(hex_cell_map, offset));
        }

        #[test]
        fn changes_nothing_if_the_offset_is_empty() {
            let offset = PixelPoint { x: 0, y: 0 };
            let cells: Vec<HexCell> = Vec::from([
                HexCell {
                    hex_coordinate: HexCellCoordinate { row: 0, column: 0 },
                    neighbor_coordinates: HashSet::new(),
                    center_point: PixelPoint { x: 1, y: 2 },
                    bounding_polygon: BoundingPolygon {
                        points: vec![PixelPoint { x: 3, y: 4 }],
                    },
                },
                HexCell {
                    hex_coordinate: HexCellCoordinate { row: 1, column: 2 },
                    neighbor_coordinates: HashSet::new(),
                    center_point: PixelPoint { x: 5, y: 6 },
                    bounding_polygon: BoundingPolygon {
                        points: vec![PixelPoint { x: 7, y: 8 }],
                    },
                },
            ]);
            let hex_cell_map: HexCellMap = cells
                .iter()
                .map(|cell| (cell.hex_coordinate, cell.clone()))
                .collect();
            let expected_hex_cell_map: HexCellMap = cells
                .into_iter()
                .map(|cell| (cell.hex_coordinate, cell))
                .collect();

            assert_eq!(expected_hex_cell_map, offset_map(hex_cell_map, offset));
        }
    }

    mod to_cell_map {
        use crate::geometry::hexagons::{HexCell, HexCellCoordinate, HexCellMap, to_cell_map};
        use crate::geometry::{BoundingPolygon, Cell, CellMap};
        use crate::{PixelBox, PixelPoint};
        use std::collections::HashSet;

        #[test]
        fn stringifies_hex_coordinates_and_adds_inscribed_rectangle() {
            let hex_cells: Vec<HexCell> = Vec::from([
                HexCell {
                    hex_coordinate: HexCellCoordinate { row: 0, column: 0 },
                    neighbor_coordinates: HashSet::from([HexCellCoordinate { row: 9, column: 10 }]),
                    center_point: PixelPoint { x: 1, y: 2 },
                    bounding_polygon: BoundingPolygon {
                        points: vec![
                            PixelPoint { x: 1, y: 1 },
                            PixelPoint { x: 2, y: 1 },
                            PixelPoint { x: 3, y: 2 },
                            PixelPoint { x: 2, y: 3 },
                            PixelPoint { x: 1, y: 3 },
                            PixelPoint { x: 0, y: 2 },
                        ],
                    },
                },
                HexCell {
                    hex_coordinate: HexCellCoordinate { row: 1, column: 2 },
                    neighbor_coordinates: HashSet::from([HexCellCoordinate {
                        row: 11,
                        column: 12,
                    }]),
                    center_point: PixelPoint { x: 5, y: 6 },
                    bounding_polygon: BoundingPolygon {
                        points: vec![
                            PixelPoint { x: 2, y: 2 },
                            PixelPoint { x: 3, y: 2 },
                            PixelPoint { x: 4, y: 3 },
                            PixelPoint { x: 3, y: 4 },
                            PixelPoint { x: 2, y: 4 },
                            PixelPoint { x: 1, y: 3 },
                        ],
                    },
                },
            ]);
            let expected_cells: Vec<Cell> = Vec::from([
                Cell {
                    coordinate: String::from("000.000"),
                    neighbor_coordinates: HashSet::from([String::from("010.009")]),
                    center_point: PixelPoint { x: 1, y: 2 },
                    bounding_polygon: BoundingPolygon {
                        points: vec![
                            PixelPoint { x: 1, y: 1 },
                            PixelPoint { x: 2, y: 1 },
                            PixelPoint { x: 3, y: 2 },
                            PixelPoint { x: 2, y: 3 },
                            PixelPoint { x: 1, y: 3 },
                            PixelPoint { x: 0, y: 2 },
                        ],
                    },
                    inscribed_rectangle: PixelBox {
                        top_left_corner: PixelPoint { x: 1, y: 1 },
                        bottom_right_corner: PixelPoint { x: 2, y: 3 },
                    },
                },
                Cell {
                    coordinate: String::from("002.001"),
                    neighbor_coordinates: HashSet::from([String::from("012.011")]),
                    center_point: PixelPoint { x: 5, y: 6 },
                    bounding_polygon: BoundingPolygon {
                        points: vec![
                            PixelPoint { x: 2, y: 2 },
                            PixelPoint { x: 3, y: 2 },
                            PixelPoint { x: 4, y: 3 },
                            PixelPoint { x: 3, y: 4 },
                            PixelPoint { x: 2, y: 4 },
                            PixelPoint { x: 1, y: 3 },
                        ],
                    },
                    inscribed_rectangle: PixelBox {
                        top_left_corner: PixelPoint { x: 2, y: 2 },
                        bottom_right_corner: PixelPoint { x: 3, y: 4 },
                    },
                },
            ]);
            let hex_cell_map: HexCellMap = hex_cells
                .into_iter()
                .map(|cell| (cell.hex_coordinate, cell))
                .collect();
            let expected_cell_map = CellMap {
                cells_by_coordinate: expected_cells
                    .into_iter()
                    .map(|cell| (cell.coordinate.clone(), cell))
                    .collect(),
            };
            assert_eq!(expected_cell_map, to_cell_map(hex_cell_map));
        }
    }

    mod compute_cell_map {
        use crate::PixelPoint;
        use crate::geometry::ComputesCellMap;
        use crate::geometry::Geometry::Hexagons;
        use crate::geometry::hexagons::fixtures::{FourByFour, HexGeometrySnapshot, ToSnapshot};
        use crate::geometry::hexagons::{offset_map, to_cell_map};

        #[test]
        fn computes_a_cell_map_for_standardized_geometries() {
            let standardized_snapshot = FourByFour::Standardized.to_snapshot();
            let geometry = Hexagons {
                definition: standardized_snapshot.geometry_definition,
            };
            let expected_cell_map = to_cell_map(standardized_snapshot.hex_cell_map);
            let no_margin = PixelPoint { x: 0, y: 0 };

            let cell_map = geometry.compute_cell_map(standardized_snapshot.dimensions, no_margin);

            assert_eq!(expected_cell_map, cell_map);
        }

        #[test]
        fn computes_a_cell_map_for_unstandardized_geometries() {
            let snapshots: Vec<HexGeometrySnapshot> = Vec::from([
                FourByFour::MustBeReflected.to_snapshot(),
                FourByFour::MustBeRotated.to_snapshot(),
                FourByFour::MustBeRotatedAndReflected.to_snapshot(),
            ]);
            let no_margin = PixelPoint { x: 0, y: 0 };
            for snapshot in snapshots {
                let geometry = Hexagons {
                    definition: snapshot.geometry_definition,
                };
                let expected_cell_map = to_cell_map(snapshot.hex_cell_map);

                let cell_map = geometry.compute_cell_map(snapshot.dimensions, no_margin);

                assert_eq!(expected_cell_map, cell_map);
            }
        }

        #[test]
        fn offsets_computed_maps_by_the_map_margin() {
            let snapshots: Vec<HexGeometrySnapshot> = Vec::from([
                FourByFour::Standardized.to_snapshot(),
                FourByFour::MustBeReflected.to_snapshot(),
                FourByFour::MustBeRotated.to_snapshot(),
                FourByFour::MustBeRotatedAndReflected.to_snapshot(),
            ]);
            let margin = PixelPoint { x: 100, y: 200 };
            for snapshot in snapshots {
                let geometry = Hexagons {
                    definition: snapshot.geometry_definition,
                };
                let expected_cell_map = to_cell_map(offset_map(snapshot.hex_cell_map, margin));

                let cell_map = geometry.compute_cell_map(snapshot.dimensions + margin * 2, margin);

                assert_eq!(expected_cell_map, cell_map);
            }
        }
    }
}
