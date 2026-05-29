use crate::PixelPoint;
use crate::geometry::hexagons::transform::InvertibleTransform;
use crate::geometry::hexagons::{
    HexCell, HexCellCoordinate, HexCellMap, HexagonGeometryDefinition,
};

#[derive(PartialEq, Debug)]
struct Identity;

impl InvertibleTransform for Identity {
    fn transform(&self, geometry: &HexagonGeometryDefinition) -> HexagonGeometryDefinition {
        geometry.clone()
    }

    fn inverse_transform_map(&self, cell_map: HexCellMap) -> HexCellMap {
        cell_map
    }

    fn inverse_transform_cell(&self, hex_cell: HexCell) -> HexCell {
        hex_cell
    }

    fn inverse_transform_point(&self, point: PixelPoint) -> PixelPoint {
        point
    }

    fn inverse_transform_coordinate(&self, coordinate: HexCellCoordinate) -> HexCellCoordinate {
        coordinate
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::geometry::hexagons::FilledTopLeftCorner;
    use crate::geometry::hexagons::FlatSides::FlatHorizontalSides;
    use FilledTopLeftCorner::FILLED;
    mod transform {
        use super::*;
        #[test]
        fn returns_a_clone_of_the_geometry() {
            let input_geometry = HexagonGeometryDefinition {
                flat_sides: FlatHorizontalSides,
                number_of_rows: 5,
                number_of_columns: 6,
                hexagon_height: 7.0,
                hexagon_width: 8.0,
                filled_top_left_corner: FILLED,
            };
        }
    }

    mod inverse_transform_map {
        use super::*;

        #[test]
        fn returns_the_unchanged_map() {
            let hex_cell_map: HexCellMap = Vec::from([HexCell {
                hex_coordinate: HexCellCoordinate { row: 0, column: 0 },
                neighbor_coordinates: vec![HexCellCoordinate { row: 1, column: 0 }],
                center_point: PixelPoint { x: 100, y: 25 },
                //Fake data, don't care about it for this test
                bounding_polygon: vec![PixelPoint { x: 75, y: 50 }],
            }])
            .into_iter()
            .map(|cell| (cell.hex_coordinate, cell))
            .collect();

            assert_eq!(
                hex_cell_map.clone(),
                Identity {}.inverse_transform_map(hex_cell_map)
            );
        }
    }

    mod inverse_transform_cell {
        use super::*;

        #[test]
        fn returns_the_unchanged_cell() {
            let cell = HexCell {
                hex_coordinate: HexCellCoordinate { row: 0, column: 0 },
                neighbor_coordinates: vec![HexCellCoordinate { row: 1, column: 0 }],
                center_point: PixelPoint { x: 100, y: 25 },
                //Fake data, don't care about it for this test
                bounding_polygon: vec![PixelPoint { x: 75, y: 50 }],
            };

            assert_eq!(cell.clone(), Identity {}.inverse_transform_cell(cell));
        }
    }

    mod inverse_transform_point {
        use super::*;

        #[test]
        fn returns_the_unchanged_point() {
            let point = PixelPoint { x: 123, y: 456 };

            assert_eq!(point, Identity {}.inverse_transform_point(point));
        }
    }

    mod inverse_transform_coordinate {
        use super::*;

        #[test]
        fn returns_the_unchanged_coordinate() {
            let coordinate = HexCellCoordinate {
                row: 123,
                column: 45,
            };

            assert_eq!(
                coordinate,
                Identity {}.inverse_transform_coordinate(coordinate)
            );
        }
    }
}
