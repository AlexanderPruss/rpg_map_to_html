use crate::geometry::Geometry::Hexagons;
use crate::{PixelBox, PixelPoint, read_input, read_input_until_valid_option};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::{BufReader, Write};
use std::path::PathBuf;

pub mod hexagons;

pub static CACHED_GEOMETRY_FILE: &str = "geometry.json";

/// Describes the structure of the RPG map. The map's [Geometry] is used to identify map cells and neighbors.
#[derive(Deserialize, Serialize, Debug, PartialEq)]
#[serde(tag = "type")]
pub enum Geometry {
    /// A hex map.
    ///
    /// Currently one where all rows have the same number of hexes, and all columns have the same number of hexes
    Hexagons {
        definition: hexagons::HexagonGeometryDefinition,
    },
    Generic {
        cell_map: CellMap,
    },
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
            Geometry::Generic { cell_map } => cell_map.clone(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct CellMap {
    pub cells_by_coordinate: HashMap<String, Cell>,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct Cell {
    pub coordinate: String,
    pub neighbor_coordinates: HashSet<String>,

    pub center_point: PixelPoint,
    pub bounding_polygon: BoundingPolygon,
    pub inscribed_rectangle: PixelBox,
}

/// A list of points defining a polygon in pixel space.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct BoundingPolygon {
    pub points: Vec<PixelPoint>,
}

/// Saves a cell map. Used primarily for caching,
/// but also allows users to edit the generated cell map manually, if they wish.
pub fn persist_cell_map_as_geometry(target_directory: &PathBuf, cell_map: CellMap) {
    let generic_geometry = Geometry::Generic { cell_map };
    let serialized = serde_json::to_string_pretty(&generic_geometry).unwrap();
    let mut path = PathBuf::from(target_directory);
    path.push(CACHED_GEOMETRY_FILE);
    let mut file = File::create(path).unwrap();
    file.write(serialized.as_bytes()).unwrap();
    file.flush().unwrap();
}

/// Loads the cell map persisted by [persist_cell_map_as_geometry]. Returns NONE if the file can't
/// be found or parsed.
pub fn load_persisted_cell_map(target_directory: &PathBuf) -> Option<CellMap> {
    let mut geometry_path = PathBuf::from(target_directory);
    geometry_path.push(CACHED_GEOMETRY_FILE);
    let geometry_file = File::open(geometry_path);
    if geometry_file.is_err() {
        return None;
    };

    let geometry_result: serde_json::Result<Geometry> =
        serde_json::from_reader(BufReader::new(geometry_file.unwrap()));
    if geometry_result.is_err() {
        return None;
    }
    match geometry_result.unwrap() {
        Hexagons { .. } => {
            //We expect a generic geometry, but don't panic; just return None and let it be recomputed
            None
        }
        Geometry::Generic { cell_map } => Some(cell_map),
    }
}

///The starting point and direction of the polygon is irrelevant for equality.
/// Polygons are equal if the points result in the same polygon.
impl PartialEq for BoundingPolygon {
    fn eq(&self, other: &Self) -> bool {
        if self.points.len() != other.points.iter().len() {
            return false;
        }
        if self.points.len() == 0 {
            return false;
        }
        let first_point = self.points.first().unwrap();
        let index_of_first_point_in_other =
            other.points.iter().position(|point| *point == *first_point);
        let mut matching_index = match index_of_first_point_in_other {
            None => return false,
            Some(index) => index,
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
            matching_index += 1;
        }
        if match_in_same_direction {
            return true;
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
                _ => matching_index - 1,
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

    //TODO: This should be Add<PixelPoint> implemented for &BoundingPolygon
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

pub(crate) fn generate_geometry_interactively() -> Geometry {
    println!("Geometry type: Generic/Hexagons (Default: Hexagons)");
    let mut input = String::new();
    match read_input_until_valid_option(&mut input, vec!["hexagons", "generic"], "hexagons")
        .as_str()
    {
        "hexagons" => hexagons::generate_geometry_interactively(),
        _ => generate_generic_geometry_interactively(),
    }
}

fn generate_generic_geometry_interactively() -> Geometry {
    println!("Generic geometries are too complicated to input interactively field by field.");
    println!("Instead, the geometry must be created ahead of time and saved in a file.");
    println!("File where the geometry is saved: (ex: target/geometry.json)");
    let mut input = String::new();
    let path = PathBuf::from(read_input(&mut input));
    serde_json::from_reader(BufReader::new(File::open(path).unwrap())).unwrap()
}

#[cfg(test)]
mod test {

    mod persist_and_load_cell_map {
        use crate::document::test::fixtures::assert_files_equal;
        use crate::geometry::{
            BoundingPolygon, Cell, CellMap, load_persisted_cell_map, persist_cell_map_as_geometry,
        };
        use crate::{PixelBox, PixelPoint};
        use std::collections::{HashMap, HashSet};
        use std::path::PathBuf;

        #[test]
        fn persists_a_cell_map_as_a_generic_geometry() {
            let mut test_case_path = PathBuf::new();
            test_case_path.push(env!("CARGO_MANIFEST_DIR"));
            test_case_path.push("test_resources");
            test_case_path.push("persist_cell_map");

            let mut target_directory = PathBuf::from(&test_case_path);
            target_directory.push("result");
            let cell_map = CellMap {
                cells_by_coordinate: HashMap::from([(
                    "abc".to_string(),
                    Cell {
                        coordinate: "abc".to_string(),
                        neighbor_coordinates: HashSet::from(["def".to_string()]),
                        center_point: PixelPoint { x: 1, y: 2 },
                        bounding_polygon: BoundingPolygon {
                            points: vec![PixelPoint { x: 2, y: 3 }, PixelPoint { x: 4, y: 5 }],
                        },
                        inscribed_rectangle: PixelBox {
                            top_left_corner: PixelPoint { x: 0, y: 1 },
                            bottom_right_corner: PixelPoint { x: 10, y: 15 },
                        },
                    },
                )]),
            };
            persist_cell_map_as_geometry(&target_directory, cell_map.clone());

            let mut result_path = PathBuf::from(&target_directory);
            result_path.push("geometry.json");
            let mut expected_path = test_case_path;
            expected_path.push("expected/geometry.json");
            assert_files_equal(&expected_path, &result_path);

            let loaded_cell_map = load_persisted_cell_map(&target_directory);
            assert_eq!(cell_map, loaded_cell_map.unwrap());
        }
    }

    mod bounding_polygon {

        mod restrict_to_bounding_box {
            use crate::geometry::BoundingPolygon;
            use crate::{PixelBox, PixelPoint};

            #[test]
            fn clamps_all_polygon_points() {
                let bounding_box = PixelBox {
                    top_left_corner: PixelPoint { x: 1, y: 1 },
                    bottom_right_corner: PixelPoint { x: 10, y: 10 },
                };
                let unclamped = PixelPoint { x: 5, y: 5 };
                let x_too_small = PixelPoint { x: 0, y: 2 };
                let x_too_small_clamped = PixelPoint { x: 1, y: 2 };
                let y_too_small = PixelPoint { x: 2, y: 0 };
                let y_too_small_clamped = PixelPoint { x: 2, y: 1 };
                let x_too_big = PixelPoint { x: 11, y: 7 };
                let x_too_big_clamped = PixelPoint { x: 10, y: 7 };
                let y_too_big = PixelPoint { x: 7, y: 11 };
                let y_too_big_clamped = PixelPoint { x: 7, y: 10 };
                let both_too_big = PixelPoint { x: 20, y: 20 };
                let both_too_big_clamped = PixelPoint { x: 10, y: 10 };
                let both_too_small = PixelPoint { x: 0, y: 0 };
                let both_too_small_clamped = PixelPoint { x: 1, y: 1 };

                let polygon = BoundingPolygon {
                    points: vec![
                        unclamped,
                        x_too_small,
                        y_too_small,
                        x_too_big,
                        y_too_big,
                        both_too_big,
                        both_too_small,
                    ],
                };
                let restricted_polygon = polygon.restrict_to_bounding_box(bounding_box);

                assert_eq!(
                    restricted_polygon,
                    BoundingPolygon {
                        points: vec![
                            unclamped,
                            x_too_small_clamped,
                            y_too_small_clamped,
                            x_too_big_clamped,
                            y_too_big_clamped,
                            both_too_big_clamped,
                            both_too_small_clamped
                        ],
                    }
                )
            }
        }

        mod offset_by {
            use crate::PixelPoint;
            use crate::geometry::BoundingPolygon;

            #[test]
            fn offsets_the_polygon_in_pixel_space() {
                let offset = PixelPoint { x: 10, y: 100 };
                let polygon = BoundingPolygon {
                    points: vec![PixelPoint { x: 1, y: 2 }, PixelPoint { x: 3, y: 4 }],
                };
                let expected = BoundingPolygon {
                    points: vec![PixelPoint { x: 11, y: 102 }, PixelPoint { x: 13, y: 104 }],
                };

                let offset_polygon = polygon.offset_by(offset);

                assert_eq!(expected, offset_polygon);
            }
        }

        mod clamp {
            use crate::PixelPoint;
            use crate::geometry::BoundingPolygon;

            #[test]
            fn clamps_with_an_implicit_lower_bound() {
                let upper_limit = PixelPoint { x: 10, y: 10 };
                let unclamped = PixelPoint { x: 5, y: 5 };
                let x_too_small = PixelPoint { x: -1, y: 2 };
                let x_too_small_clamped = PixelPoint { x: 0, y: 2 };
                let y_too_small = PixelPoint { x: -2, y: 1 };
                let y_too_small_clamped = PixelPoint { x: 0, y: 1 };
                let x_too_big = PixelPoint { x: 11, y: 7 };
                let x_too_big_clamped = PixelPoint { x: 10, y: 7 };
                let y_too_big = PixelPoint { x: 7, y: 11 };
                let y_too_big_clamped = PixelPoint { x: 7, y: 10 };
                let both_too_big = PixelPoint { x: 20, y: 20 };
                let both_too_big_clamped = PixelPoint { x: 10, y: 10 };
                let both_too_small = PixelPoint { x: -1, y: -12 };
                let both_too_small_clamped = PixelPoint { x: 0, y: 0 };

                let polygon = BoundingPolygon {
                    points: vec![
                        unclamped,
                        x_too_small,
                        y_too_small,
                        x_too_big,
                        y_too_big,
                        both_too_big,
                        both_too_small,
                    ],
                };
                let clamped_polygon = polygon.clamp(upper_limit);

                assert_eq!(
                    clamped_polygon,
                    BoundingPolygon {
                        points: vec![
                            unclamped,
                            x_too_small_clamped,
                            y_too_small_clamped,
                            x_too_big_clamped,
                            y_too_big_clamped,
                            both_too_big_clamped,
                            both_too_small_clamped
                        ],
                    }
                )
            }
        }
    }

    mod equality {
        use crate::PixelPoint;
        use crate::geometry::BoundingPolygon;

        #[test]
        fn equals_polygons_with_identical_point_lists() {
            let polygon = BoundingPolygon {
                points: vec![
                    PixelPoint { x: 1, y: 2 },
                    PixelPoint { x: 3, y: 4 },
                    PixelPoint { x: 5, y: 6 },
                ],
            };
            let identical = BoundingPolygon {
                points: vec![
                    PixelPoint { x: 1, y: 2 },
                    PixelPoint { x: 3, y: 4 },
                    PixelPoint { x: 5, y: 6 },
                ],
            };
            assert_eq!(polygon, identical);
        }

        #[test]
        fn equals_polygons_with_identical_points_but_a_different_starting_point() {
            let polygon = BoundingPolygon {
                points: vec![
                    PixelPoint { x: 1, y: 2 },
                    PixelPoint { x: 3, y: 4 },
                    PixelPoint { x: 5, y: 6 },
                ],
            };
            let shifted_polygon = BoundingPolygon {
                points: vec![
                    PixelPoint { x: 3, y: 4 },
                    PixelPoint { x: 5, y: 6 },
                    PixelPoint { x: 1, y: 2 },
                ],
            };
            assert_eq!(polygon, shifted_polygon);
        }

        #[test]
        fn equals_polygons_with_identical_point_order_but_reversed() {
            let polygon = BoundingPolygon {
                points: vec![
                    PixelPoint { x: 1, y: 2 },
                    PixelPoint { x: 3, y: 4 },
                    PixelPoint { x: 5, y: 6 },
                ],
            };
            let shifted_reversed_polygon = BoundingPolygon {
                points: vec![
                    PixelPoint { x: 3, y: 4 },
                    PixelPoint { x: 1, y: 2 },
                    PixelPoint { x: 5, y: 6 },
                ],
            };
            assert_eq!(polygon, shifted_reversed_polygon);
        }

        #[test]
        fn does_not_equal_polygons_with_a_different_size() {
            let polygon = BoundingPolygon {
                points: vec![
                    PixelPoint { x: 1, y: 2 },
                    PixelPoint { x: 3, y: 4 },
                    PixelPoint { x: 5, y: 6 },
                ],
            };
            let missing_last_point = BoundingPolygon {
                points: vec![PixelPoint { x: 1, y: 2 }, PixelPoint { x: 3, y: 4 }],
            };
            assert_ne!(polygon, missing_last_point);
        }

        #[test]
        fn does_not_equal_polygons_with_different_points() {
            let polygon = BoundingPolygon {
                points: vec![
                    PixelPoint { x: 1, y: 2 },
                    PixelPoint { x: 3, y: 4 },
                    PixelPoint { x: 5, y: 6 },
                ],
            };
            let last_point_different = BoundingPolygon {
                points: vec![
                    PixelPoint { x: 1, y: 2 },
                    PixelPoint { x: 3, y: 4 },
                    PixelPoint { x: 66, y: 55 },
                ],
            };
            assert_ne!(polygon, last_point_different);
        }

        #[test]
        fn does_not_equal_polygons_with_identical_points_in_a_different_iteration_order() {
            let polygon = BoundingPolygon {
                points: vec![
                    PixelPoint { x: 1, y: 2 },
                    PixelPoint { x: 3, y: 4 },
                    PixelPoint { x: 5, y: 6 },
                ],
            };
            let wrong_order = BoundingPolygon {
                points: vec![
                    PixelPoint { x: 1, y: 2 },
                    PixelPoint { x: 5, y: 6 },
                    PixelPoint { x: 3, y: 4 },
                ],
            };
            assert_eq!(polygon, wrong_order);
        }

        #[test]
        fn does_not_panic_when_either_list_is_empty() {
            let polygon = BoundingPolygon {
                points: vec![
                    PixelPoint { x: 1, y: 2 },
                    PixelPoint { x: 3, y: 4 },
                    PixelPoint { x: 5, y: 6 },
                ],
            };
            let empty_polygon = BoundingPolygon { points: vec![] };

            assert_ne!(empty_polygon, polygon);
            assert_ne!(polygon, empty_polygon);
        }
    }
}
