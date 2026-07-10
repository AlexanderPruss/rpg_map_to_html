use crate::PixelPoint;
use crate::geometry::hexagons::standardized::{
    OffsetNeighborCoordinate, OffsetNeighborCoordinates,
};
use crate::geometry::hexagons::{
    FilledTopLeftCorner, FlatSides, HexCell, HexCellCoordinate, HexCellMap,
    HexagonGeometryDefinition, to_cell_map,
};
use crate::geometry::{BoundingPolygon, Cell, CellMap};

pub enum FourByFour {
    Standardized,
    MustBeReflected,
    MustBeRotatedAndReflected,
    MustBeRotated,
}
impl FourByFour {
    fn create_cell(&self, hex_coordinate: HexCellCoordinate) -> HexCell {
        let maximum_hex_coordinate = HexCellCoordinate { row: 3, column: 3 };
        let (
            center_point,
            cell_0_0_center,
            bounding_polygon_around_cell_0_0,
            even_odd_neighbors_of_0_0,
        ) = match self {
            FourByFour::Standardized => {
                let center_point = PixelPoint {
                    x: 50 + hex_coordinate.column as i32 * 75,
                    y: 25 + hex_coordinate.row as i32 * 50 + hex_coordinate.column as i32 % 2 * 25,
                };
                let cell_0_0_center = PixelPoint { x: 50, y: 25 };
                let bounding_polygon_around_cell_0_0 = BoundingPolygon {
                    points: Vec::from([
                        PixelPoint { x: 75, y: 0 },
                        PixelPoint { x: 100, y: 25 },
                        PixelPoint { x: 75, y: 50 },
                        PixelPoint { x: 25, y: 50 },
                        PixelPoint { x: 0, y: 25 },
                        PixelPoint { x: 25, y: 0 },
                    ]),
                };
                let odd_neighbors_of_0_0 = OffsetNeighborCoordinates {
                    coordinates: vec![
                        OffsetNeighborCoordinate { row: -1, column: 0 },
                        OffsetNeighborCoordinate { row: -1, column: 1 },
                        OffsetNeighborCoordinate { row: 0, column: 1 },
                        OffsetNeighborCoordinate { row: 1, column: 0 },
                        OffsetNeighborCoordinate { row: 0, column: -1 },
                        OffsetNeighborCoordinate {
                            row: -1,
                            column: -1,
                        },
                    ],
                    maximum_coordinate: maximum_hex_coordinate,
                };
                let even_neighbors_of_0_0 = OffsetNeighborCoordinates {
                    coordinates: vec![
                        OffsetNeighborCoordinate { row: -1, column: 0 },
                        OffsetNeighborCoordinate { row: 0, column: 1 },
                        OffsetNeighborCoordinate { row: 1, column: 1 },
                        OffsetNeighborCoordinate { row: 1, column: 0 },
                        OffsetNeighborCoordinate { row: 1, column: -1 },
                        OffsetNeighborCoordinate { row: 0, column: -1 },
                    ],
                    maximum_coordinate: maximum_hex_coordinate,
                };
                (
                    center_point,
                    cell_0_0_center,
                    bounding_polygon_around_cell_0_0,
                    (even_neighbors_of_0_0, odd_neighbors_of_0_0),
                )
            }
            FourByFour::MustBeReflected => {
                let center_point = PixelPoint {
                    x: 50 + hex_coordinate.column as i32 * 75,
                    y: 50 + hex_coordinate.row as i32 * 50 - hex_coordinate.column as i32 % 2 * 25,
                };
                let cell_0_0_center = PixelPoint { x: 50, y: 50 };
                let bounding_polygon_around_cell_0_0 = BoundingPolygon {
                    points: Vec::from([
                        PixelPoint { x: 75, y: 25 },
                        PixelPoint { x: 100, y: 50 },
                        PixelPoint { x: 75, y: 75 },
                        PixelPoint { x: 25, y: 75 },
                        PixelPoint { x: 0, y: 50 },
                        PixelPoint { x: 25, y: 25 },
                    ]),
                };
                let odd_neighbors_of_0_0 = OffsetNeighborCoordinates {
                    coordinates: vec![
                        OffsetNeighborCoordinate { row: -1, column: 0 },
                        OffsetNeighborCoordinate { row: 0, column: 1 },
                        OffsetNeighborCoordinate { row: 1, column: 1 },
                        OffsetNeighborCoordinate { row: 1, column: 0 },
                        OffsetNeighborCoordinate { row: 1, column: -1 },
                        OffsetNeighborCoordinate { row: 0, column: -1 },
                    ],
                    maximum_coordinate: maximum_hex_coordinate,
                };
                let even_neighbors_of_0_0 = OffsetNeighborCoordinates {
                    coordinates: vec![
                        OffsetNeighborCoordinate { row: -1, column: 0 },
                        OffsetNeighborCoordinate { row: -1, column: 1 },
                        OffsetNeighborCoordinate { row: 0, column: 1 },
                        OffsetNeighborCoordinate { row: 1, column: 0 },
                        OffsetNeighborCoordinate { row: 0, column: -1 },
                        OffsetNeighborCoordinate {
                            row: -1,
                            column: -1,
                        },
                    ],
                    maximum_coordinate: maximum_hex_coordinate,
                };
                (
                    center_point,
                    cell_0_0_center,
                    bounding_polygon_around_cell_0_0,
                    (even_neighbors_of_0_0, odd_neighbors_of_0_0),
                )
            }
            FourByFour::MustBeRotatedAndReflected => {
                let center_point = PixelPoint {
                    x: 25 + hex_coordinate.column as i32 * 50 + hex_coordinate.row as i32 % 2 * 25,
                    y: 50 + hex_coordinate.row as i32 * 75,
                };
                let cell_0_0_center = PixelPoint { x: 25, y: 50 };
                let bounding_polygon_around_cell_0_0 = BoundingPolygon {
                    points: Vec::from([
                        PixelPoint { x: 50, y: 25 },
                        PixelPoint { x: 50, y: 75 },
                        PixelPoint { x: 25, y: 100 },
                        PixelPoint { x: 0, y: 75 },
                        PixelPoint { x: 0, y: 25 },
                        PixelPoint { x: 25, y: 0 },
                    ]),
                };
                let even_neighbors_of_0_0 = OffsetNeighborCoordinates {
                    coordinates: vec![
                        OffsetNeighborCoordinate { row: -1, column: 0 },
                        OffsetNeighborCoordinate { row: -1, column: 1 },
                        OffsetNeighborCoordinate { row: 0, column: 1 },
                        OffsetNeighborCoordinate { row: 1, column: 1 },
                        OffsetNeighborCoordinate { row: 1, column: 0 },
                        OffsetNeighborCoordinate { row: 0, column: -1 },
                    ],
                    maximum_coordinate: maximum_hex_coordinate,
                };
                let odd_neighbors_of_0_0 = OffsetNeighborCoordinates {
                    coordinates: vec![
                        OffsetNeighborCoordinate {
                            row: -1,
                            column: -1,
                        },
                        OffsetNeighborCoordinate { row: -1, column: 0 },
                        OffsetNeighborCoordinate { row: 0, column: 1 },
                        OffsetNeighborCoordinate { row: 1, column: 0 },
                        OffsetNeighborCoordinate { row: 1, column: -1 },
                        OffsetNeighborCoordinate { row: 0, column: -1 },
                    ],
                    maximum_coordinate: maximum_hex_coordinate,
                };
                (
                    center_point,
                    cell_0_0_center,
                    bounding_polygon_around_cell_0_0,
                    (even_neighbors_of_0_0, odd_neighbors_of_0_0),
                )
            }
            FourByFour::MustBeRotated => {
                let center_point = PixelPoint {
                    x: 50 + hex_coordinate.column as i32 * 50 - hex_coordinate.row as i32 % 2 * 25,
                    y: 50 + hex_coordinate.row as i32 * 75,
                };
                let cell_0_0_center = PixelPoint { x: 50, y: 50 };
                let bounding_polygon_around_cell_0_0 = BoundingPolygon {
                    points: Vec::from([
                        PixelPoint { x: 75, y: 25 },
                        PixelPoint { x: 75, y: 75 },
                        PixelPoint { x: 50, y: 100 },
                        PixelPoint { x: 25, y: 75 },
                        PixelPoint { x: 25, y: 25 },
                        PixelPoint { x: 50, y: 0 },
                    ]),
                };
                let even_neighbors_of_0_0 = OffsetNeighborCoordinates {
                    coordinates: vec![
                        OffsetNeighborCoordinate {
                            row: -1,
                            column: -1,
                        },
                        OffsetNeighborCoordinate { row: -1, column: 0 },
                        OffsetNeighborCoordinate { row: 0, column: 1 },
                        OffsetNeighborCoordinate { row: 1, column: 0 },
                        OffsetNeighborCoordinate { row: 1, column: -1 },
                        OffsetNeighborCoordinate { row: 0, column: -1 },
                    ],
                    maximum_coordinate: maximum_hex_coordinate,
                };
                let odd_neighbors_of_0_0 = OffsetNeighborCoordinates {
                    coordinates: vec![
                        OffsetNeighborCoordinate { row: -1, column: 0 },
                        OffsetNeighborCoordinate { row: -1, column: 1 },
                        OffsetNeighborCoordinate { row: 0, column: 1 },
                        OffsetNeighborCoordinate { row: 1, column: 1 },
                        OffsetNeighborCoordinate { row: 1, column: 0 },
                        OffsetNeighborCoordinate { row: 0, column: -1 },
                    ],
                    maximum_coordinate: maximum_hex_coordinate,
                };
                (
                    center_point,
                    cell_0_0_center,
                    bounding_polygon_around_cell_0_0,
                    (even_neighbors_of_0_0, odd_neighbors_of_0_0),
                )
            }
        };

        let neighbors_to_use = match self {
            FourByFour::Standardized | FourByFour::MustBeReflected => {
                if hex_coordinate.column % 2 == 1 {
                    even_odd_neighbors_of_0_0.0
                } else {
                    even_odd_neighbors_of_0_0.1
                }
            }
            FourByFour::MustBeRotatedAndReflected | FourByFour::MustBeRotated => {
                if hex_coordinate.row % 2 == 1 {
                    even_odd_neighbors_of_0_0.0
                } else {
                    even_odd_neighbors_of_0_0.1
                }
            }
        };

        let neighbor_coordinates = &neighbors_to_use + hex_coordinate;
        //        println!("Row: {}, Column: {}, number of coords: {}", hex_coordinate.row, hex_coordinate.column, neighbor_coordinates.len());
        HexCell {
            hex_coordinate,
            neighbor_coordinates: neighbor_coordinates,
            center_point,
            bounding_polygon: bounding_polygon_around_cell_0_0
                .offset_by(center_point - cell_0_0_center),
        }
    }
}

/// Helpful for debugging test failures, since it fails on the concrete cell that doesn't map properly.
pub(super) fn _assert_hex_cells_equal(expected: HexCellMap, actual: HexCellMap) {
    let mut expected_cells: Vec<HexCell> = expected
        .into_iter()
        .map(|(_coordinate, value)| value)
        .collect();
    expected_cells.sort_by(|first, second| first.hex_coordinate.cmp(&second.hex_coordinate));
    let mut actual_cells: Vec<HexCell> = actual
        .into_iter()
        .map(|(_coordinate, value)| value)
        .collect();
    actual_cells.sort_by(|first, second| first.hex_coordinate.cmp(&second.hex_coordinate));
    for i in 0..15 {
        let expected_cell = &expected_cells[i];
        let actual_cell = &actual_cells[i];
        assert_eq!(*expected_cell, *actual_cell);
    }
    assert_eq!(expected_cells, actual_cells);
}

/// Helpful for debugging test failures, since it fails on the concrete cell that doesn't map properly.
pub fn _assert_cells_equal(expected: CellMap, actual: CellMap) {
    let mut expected_cells: Vec<Cell> = expected
        .cells_by_coordinate
        .into_iter()
        .map(|(_coordinate, value)| value)
        .collect();
    expected_cells.sort_by(|first, second| first.coordinate.cmp(&second.coordinate));
    let mut actual_cells: Vec<Cell> = actual
        .cells_by_coordinate
        .into_iter()
        .map(|(_coordinate, value)| value)
        .collect();
    actual_cells.sort_by(|first, second| first.coordinate.cmp(&second.coordinate));
    for i in 0..15 {
        let expected_cell = &expected_cells[i];
        let actual_cell = &actual_cells[i];
        assert_eq!(*expected_cell, *actual_cell);
    }
    assert_eq!(expected_cells, actual_cells);
}

pub struct HexGeometrySnapshot {
    pub dimensions: PixelPoint,
    pub geometry_definition: HexagonGeometryDefinition,
    pub(super) hex_cell_map: HexCellMap,
    pub cell_map: CellMap,
}

pub trait ToSnapshot {
    fn to_snapshot(&self) -> HexGeometrySnapshot;
}

impl ToSnapshot for FourByFour {
    fn to_snapshot(&self) -> HexGeometrySnapshot {
        let mut cells: Vec<HexCell> = Vec::new();
        for row in 0..4 {
            for column in 0..4 {
                cells.push(self.create_cell(HexCellCoordinate { row, column }));
            }
        }
        let hex_cell_map: HexCellMap = cells
            .into_iter()
            .map(|cell| (cell.hex_coordinate, cell))
            .collect();
        match self {
            FourByFour::Standardized => HexGeometrySnapshot {
                dimensions: PixelPoint { x: 325, y: 225 },
                geometry_definition: HexagonGeometryDefinition {
                    flat_sides: FlatSides::FlatHorizontalSides,
                    number_of_rows: 4,
                    number_of_columns: 4,
                    hexagon_height: 50.0,
                    hexagon_width: 100.0,
                    filled_top_left_corner: FilledTopLeftCorner::FILLED,
                },
                hex_cell_map: hex_cell_map.clone(),
                cell_map: to_cell_map(hex_cell_map),
            },
            FourByFour::MustBeReflected => HexGeometrySnapshot {
                dimensions: PixelPoint { x: 325, y: 225 },
                geometry_definition: HexagonGeometryDefinition {
                    flat_sides: FlatSides::FlatHorizontalSides,
                    number_of_rows: 4,
                    number_of_columns: 4,
                    hexagon_height: 50.0,
                    hexagon_width: 100.0,
                    filled_top_left_corner: FilledTopLeftCorner::EMPTY,
                },
                hex_cell_map: hex_cell_map.clone(),
                cell_map: to_cell_map(hex_cell_map),
            },
            FourByFour::MustBeRotatedAndReflected => HexGeometrySnapshot {
                dimensions: PixelPoint { x: 225, y: 325 },
                geometry_definition: HexagonGeometryDefinition {
                    flat_sides: FlatSides::FlatVerticalSides,
                    number_of_rows: 4,
                    number_of_columns: 4,
                    hexagon_height: 100.0,
                    hexagon_width: 50.0,
                    filled_top_left_corner: FilledTopLeftCorner::FILLED,
                },
                hex_cell_map: hex_cell_map.clone(),
                cell_map: to_cell_map(hex_cell_map),
            },
            FourByFour::MustBeRotated => HexGeometrySnapshot {
                dimensions: PixelPoint { x: 225, y: 325 },
                geometry_definition: HexagonGeometryDefinition {
                    flat_sides: FlatSides::FlatVerticalSides,
                    number_of_rows: 4,
                    number_of_columns: 4,
                    hexagon_height: 100.0,
                    hexagon_width: 50.0,
                    filled_top_left_corner: FilledTopLeftCorner::EMPTY,
                },
                hex_cell_map: hex_cell_map.clone(),
                cell_map: to_cell_map(hex_cell_map),
            },
        }
    }
}
