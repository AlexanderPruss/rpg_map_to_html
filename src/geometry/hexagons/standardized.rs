use crate::geometry::BoundingPolygon;
use crate::geometry::hexagons::transform::{InvertibleStandardizedGeometry, InvertibleTransform};
use crate::geometry::hexagons::{HexCell, HexCellCoordinate, HexCellMap};
use crate::{PixelPoint, PositionDelta};
use std::collections::{HashMap, HashSet};
use std::ops::Add;

/// A hex geometry in standard form, meaning that it has flat horizontal sides and the top-left
/// corner is filled.
/// Ex:
///``` picture
///             ••••••••••
///            •          •
///           •            •
///          •              ••••••••••
///           •            •          •
///            •          •            •
///             ••••••••••              •
///            •          •            •
///           •            •          •
///          •              ••••••••••
///           •            •
///            •          •
///             ••••••••••
/// ```
#[derive(PartialEq, Debug)]
pub struct StandardizedHexGeometryDefinition {
    pub number_of_rows: u8,
    pub number_of_columns: u8,
    pub hexagon_height: f32,
    pub hexagon_width: f32,
    pub geometry_dimensions: PixelPoint,
}

pub struct StandardizedHexCellMap {
    standardized_map: HexCellMap,
    transforms_applied: Vec<Box<dyn InvertibleTransform>>,
}

impl StandardizedHexCellMap {
    /// Consumes the [StandardizedHexCellMap] by inverting the transforms that standardized it.
    ///
    /// This results in a [HexCellMap] for the original geometry.
    pub fn invert_standardization(self) -> HexCellMap {
        self.transforms_applied.iter().rev().fold(
            self.standardized_map,
            |current_map, invertible_transform| {
                invertible_transform.inverse_transform_map(current_map)
            },
        )
    }
}

impl InvertibleStandardizedGeometry {
    /// Computes a Hex Cell map, using the assumptions that the Hex Geometry is a
    /// [StandardizedHexGeometryDefinition] with no margins.
    pub fn compute_standardized_cell_map(self) -> StandardizedHexCellMap {
        let hex_height = self.standardized_geometry.compute_hex_height();
        let hex_widths = self.standardized_geometry.compute_hex_width(hex_height);
        let (hex_width, hex_middle_width, hex_edge_with) = (
            hex_widths.hex_width,
            hex_widths.hex_middle_width,
            hex_widths.hex_edge_width,
        );

        let hex_0_0_center = PositionDelta {
            x: hex_width / 2.0,
            y: hex_height / 2.0,
        };
        let row_position_delta = PositionDelta {
            x: 0.0,
            y: hex_height,
        };
        let column_position_delta = PositionDelta {
            x: hex_middle_width + hex_edge_with,
            y: 0.0,
        };
        let odd_column_offset = PositionDelta {
            x: 0.0,
            y: hex_height / 2.0,
        };

        let bounding_polygon_centered_on_hex_0_0 =
            Self::compute_bounding_polygon_around_hex_0_0(hex_height, hex_widths);
        let maximum_coordinate = HexCellCoordinate {
            row: self.standardized_geometry.number_of_rows - 1,
            column: self.standardized_geometry.number_of_columns - 1,
        };
        let (even_column_neighbor_offsets, odd_column_neighbor_offsets) =
            Self::compute_offset_neighbor_coordinates(maximum_coordinate);

        let mut hex_cell_map: HashMap<HexCellCoordinate, HexCell> = HashMap::new();
        for row_coordinate in 0..self.standardized_geometry.number_of_rows {
            for column_coordinate in 0..self.standardized_geometry.number_of_columns {
                let hex_coordinate = HexCellCoordinate {
                    row: row_coordinate,
                    column: column_coordinate,
                };
                let center_point: PixelPoint = PixelPoint::from(
                    hex_0_0_center
                        + row_position_delta * row_coordinate
                        + column_position_delta * column_coordinate
                        + odd_column_offset * (column_coordinate % 2),
                );
                let neighbor_coordinates = if column_coordinate % 2 == 0 {
                    &even_column_neighbor_offsets + hex_coordinate
                } else {
                    &odd_column_neighbor_offsets + hex_coordinate
                };
                let cell = HexCell {
                    hex_coordinate,
                    center_point,
                    neighbor_coordinates,
                    bounding_polygon: bounding_polygon_centered_on_hex_0_0
                        .offset_by(center_point - hex_0_0_center)
                        .clamp(self.standardized_geometry.geometry_dimensions),
                };

                let inserted = hex_cell_map.insert(hex_coordinate, cell);
                assert_eq!(inserted, None);
            }
        }
        StandardizedHexCellMap {
            standardized_map: hex_cell_map,
            transforms_applied: self.transforms_applied,
        }
    }

    fn compute_bounding_polygon_around_hex_0_0(
        hex_height: f32,
        hex_widths: HexWidths,
    ) -> BoundingPolygon {
        let hex_width = hex_widths.hex_width as i32;
        let hex_edge_width = hex_widths.hex_edge_width as i32;
        let hex_height = hex_height as i32;
        let bounding_polygon_centered_on_hex_0_0 = BoundingPolygon {
            points: Vec::from([
                PixelPoint {
                    x: hex_width - hex_edge_width,
                    y: 0,
                },
                PixelPoint {
                    x: hex_width,
                    y: hex_height / 2,
                },
                PixelPoint {
                    x: hex_width - hex_edge_width,
                    y: hex_height,
                },
                PixelPoint {
                    x: hex_edge_width,
                    y: hex_height,
                },
                PixelPoint {
                    x: 0,
                    y: hex_height / 2,
                },
                PixelPoint {
                    x: hex_edge_width,
                    y: 0,
                },
            ]),
        };
        bounding_polygon_centered_on_hex_0_0
    }

    fn compute_offset_neighbor_coordinates(
        maximum_coordinate: HexCellCoordinate,
    ) -> (OffsetNeighborCoordinates, OffsetNeighborCoordinates) {
        let even_column_neighbor_offsets = OffsetNeighborCoordinates {
            coordinates: Vec::from([
                OffsetNeighborCoordinate { row: -1, column: 0 },
                OffsetNeighborCoordinate { row: -1, column: 1 },
                OffsetNeighborCoordinate { row: 0, column: 1 },
                OffsetNeighborCoordinate { row: 1, column: 0 },
                OffsetNeighborCoordinate { row: 0, column: -1 },
                OffsetNeighborCoordinate {
                    row: -1,
                    column: -1,
                },
            ]),
            maximum_coordinate,
        };
        let odd_column_neighbor_offsets = OffsetNeighborCoordinates {
            coordinates: Vec::from([
                OffsetNeighborCoordinate { row: -1, column: 0 },
                OffsetNeighborCoordinate { row: 0, column: 1 },
                OffsetNeighborCoordinate { row: 1, column: 1 },
                OffsetNeighborCoordinate { row: 1, column: 0 },
                OffsetNeighborCoordinate { row: 1, column: -1 },
                OffsetNeighborCoordinate { row: 0, column: -1 },
            ]),
            maximum_coordinate,
        };
        (even_column_neighbor_offsets, odd_column_neighbor_offsets)
    }
}

#[derive(Debug, PartialEq)]
pub(super) struct OffsetNeighborCoordinates {
    pub coordinates: Vec<OffsetNeighborCoordinate>,
    pub maximum_coordinate: HexCellCoordinate,
}
#[derive(Debug, PartialEq, Clone, Copy)]
pub(super) struct OffsetNeighborCoordinate {
    pub row: i16,
    pub column: i16,
}
impl Add<HexCellCoordinate> for &OffsetNeighborCoordinates {
    type Output = HashSet<HexCellCoordinate>;

    fn add(self, rhs: HexCellCoordinate) -> HashSet<HexCellCoordinate> {
        self.coordinates
            .iter()
            .map(|offset| self.offset_coordinate_if_valid(offset, rhs))
            .filter(|option| option.is_some())
            .map(|option| option.unwrap())
            .collect()
    }
}

impl OffsetNeighborCoordinates {
    /// Only non-negative coordinates within the allowed [maximum_coordinate] are allowed.
    fn offset_coordinate_if_valid(
        &self,
        offset: &OffsetNeighborCoordinate,
        coordinate: HexCellCoordinate,
    ) -> Option<HexCellCoordinate> {
        let row_candidate = coordinate.row as i16 + offset.row;
        let column_candidate = coordinate.column as i16 + offset.column;
        if row_candidate < 0
            || row_candidate > self.maximum_coordinate.row as i16
            || column_candidate < 0
            || column_candidate > self.maximum_coordinate.column as i16
        {
            return None;
        }
        Some(HexCellCoordinate {
            row: row_candidate as u8,
            column: column_candidate as u8,
        })
    }
}

impl StandardizedHexGeometryDefinition {
    /// The first row adds three halves full cell height, each further row adds one more.
    ///
    /// H = 3*h/2 + (row-1)*h.
    ///
    /// Solving for h,
    ///
    /// h = 2H/(1+2*rows)
    ///
    ///``` picture
    ///                  ┌─────╴     ••••••••••                 ╶──────┐
    ///                  │          •          •                       │
    ///                  │         •            •                      │
    ///  h - cell height │        •              ••••••••••            │
    ///                  │         •            •          •           │
    ///                  │          •          •            •          │
    ///                  └─────╴     ••••••••••              •         │
    ///                             •          •            •          │  H - total height
    ///                            •            •          •           │
    ///                           •              ••••••••••            │
    ///                            •            •                      │
    ///                             •          •                       │
    ///                              ••••••••••                 ╶──────┘
    ///```
    fn compute_hex_height(&self) -> f32 {
        let total_height = self.geometry_dimensions.y as f32;
        (2.0 * total_height) / (2.0 * self.number_of_rows as f32 + 1.0)
    }

    /// The ratio of width to height is part of the geometry definition, so computing one from the
    /// other is trivial. We also compute two other widths:
    ///```picture
    ///                         W:= map width
    ///         ┌──────────────────────────────────────────┐
    ///         │                                          │
    ///         │  •••••••••••               •••••••••••   │
    ///         │ •           •             •           •  │
    ///         ╵•             •           •             • ╵
    ///         •       •       •••••••••••       •       •
    ///          •      ╷      •           •             •
    ///           •     │     •             •           •
    ///            •••••│•••••       •       •••••••••••
    ///                 │     •      ╷      •
    ///                 │      •     │     •
    ///                 │       •••••│•••••
    ///                 │            │
    ///                 └────────────┘
    ///                    w_delta := x delta whenever we shift over a column
    ///
    ///                             w_mid := width of the straight part of the hexagon
    ///                          ┌─────────┐
    ///                          ╵         ╵
    ///                          •••••••••••
    ///                         •           •
    ///                        •             •
    ///                       •       •       •
    ///                       ╷•             •
    ///                       │ •           •
    ///                       │  •••••••••••
    ///                       │  ╷
    ///                       │  │
    ///                       └──┘
    ///                        w_edge := width of the slanted part of the hexagon
    ///```
    ///
    /// With the relations
    ///
    /// ```equation
    ///     w_mid + 2*w_edge = w_hex
    ///     w_hex + (1-#cols)*(w_edge + w_mid) = W
    ///     (implicit: w_delta = w_mid + 2*w_edge)
    /// ```
    ///
    /// Once we habe w_hex, we can compute w_edge and w_mid and use these for building the cell map.
    fn compute_hex_width(&self, hex_height: f32) -> HexWidths {
        let hex_width = hex_height * self.hexagon_width / self.hexagon_height;
        let hex_edge_width = (self.geometry_dimensions.x as f32
            - self.number_of_columns as f32 * hex_width)
            / (1.0 - self.number_of_columns as f32);
        let hex_middle_width = hex_width - 2.0 * hex_edge_width;
        if hex_width < 0.0 || hex_middle_width < 0.0 || hex_edge_width < 0.0 {
            panic!(
                "Computed negative hex widths. This means the input geometry and dimensions are not consistent.\n (hex_width, hex_edge_with, hex_middle_width): ({}, {}, {})",
                hex_width, hex_edge_width, hex_middle_width
            )
        }
        HexWidths {
            hex_width,
            hex_middle_width,
            hex_edge_width,
        }
    }
}

///```picture
///                w_mid := width of the straight part of the hexagon
///             ┌─────────┐
///             ╵         ╵
///             •••••••••••
///            •           •
///           •             •
///          •       •       •
///          ╷•             •
///          │ •           •
///          │  •••••••••••
///          │  ╷
///          │  │
///          └──┘
///           w_edge := width of the slanted part of the hexagon
/// ```
#[derive(Debug, PartialEq)]
struct HexWidths {
    hex_width: f32,
    hex_middle_width: f32,
    hex_edge_width: f32,
}

#[cfg(test)]
mod test {
    use super::*;

    mod standardized_hex_cell_map {
        use crate::geometry::hexagons::standardized::StandardizedHexCellMap;
        use crate::geometry::hexagons::test::fixtures::{FourByFour, ToSnapshot};
        use crate::geometry::hexagons::transform::identity::Identity;
        use crate::geometry::hexagons::transform::reflect::ReflectOverXAxis;
        use crate::geometry::hexagons::transform::rotate::RotateCounterClockwise;

        #[test]
        fn inverts_its_hex_map_by_reversing_the_applied_transforms_in_reverse_order() {
            let standardized_snapshot = FourByFour::Standardized.to_snapshot();
            let must_be_reflected_and_rotated_snapshot =
                FourByFour::MustBeRotatedAndReflected.to_snapshot();
            let (rotation, rotated_geometry) = RotateCounterClockwise::rotate(
                must_be_reflected_and_rotated_snapshot.dimensions,
                &must_be_reflected_and_rotated_snapshot.geometry_definition,
            );
            let reflection =
                ReflectOverXAxis::reflect(rotation.rotated_map_dimensions, &rotated_geometry).0;
            let standardized_hex_cell_map = StandardizedHexCellMap {
                standardized_map: standardized_snapshot.hex_cell_map,
                transforms_applied: vec![
                    Box::new(Identity {}),
                    Box::new(rotation),
                    Box::new(reflection),
                ],
            };

            let inverted_map = standardized_hex_cell_map.invert_standardization();

            assert_eq!(
                must_be_reflected_and_rotated_snapshot.hex_cell_map,
                inverted_map
            );
        }
    }

    mod invertible_standardized_geometry {
        use crate::geometry::hexagons::HexCellCoordinate;
        use crate::geometry::hexagons::standardized::{
            HexWidths, StandardizedHexGeometryDefinition,
        };
        use crate::geometry::hexagons::test::fixtures::{FourByFour, ToSnapshot};
        use crate::geometry::hexagons::transform::identity::Identity;
        use crate::geometry::hexagons::transform::reflect::ReflectOverXAxis;
        use crate::geometry::hexagons::transform::rotate::RotateCounterClockwise;
        use crate::geometry::hexagons::transform::transform_equality::assert_transforms_equal;
        use crate::geometry::hexagons::transform::{
            InvertibleStandardizedGeometry, InvertibleTransform,
        };

        #[test]
        fn computes_cell_maps_for_standardized_geometries() {
            let standardized_snapshot = FourByFour::Standardized.to_snapshot();
            let must_be_reflected_and_rotated_snapshot =
                FourByFour::MustBeRotatedAndReflected.to_snapshot();
            let (expected_rotation, rotated_geometry) = RotateCounterClockwise::rotate(
                must_be_reflected_and_rotated_snapshot.dimensions,
                &must_be_reflected_and_rotated_snapshot.geometry_definition,
            );
            let expected_reflection = ReflectOverXAxis::reflect(
                expected_rotation.rotated_map_dimensions,
                &rotated_geometry,
            )
            .0;
            let expected_transforms: Vec<Box<dyn InvertibleTransform>> = vec![
                Box::new(Identity {}),
                Box::new(expected_rotation.clone()),
                Box::new(expected_reflection.clone()),
            ];
            let invertible_geometry = InvertibleStandardizedGeometry {
                standardized_geometry: StandardizedHexGeometryDefinition {
                    number_of_rows: standardized_snapshot.geometry_definition.number_of_rows,
                    number_of_columns: standardized_snapshot.geometry_definition.number_of_columns,
                    hexagon_height: standardized_snapshot.geometry_definition.hexagon_height,
                    hexagon_width: standardized_snapshot.geometry_definition.hexagon_width,
                    geometry_dimensions: standardized_snapshot.dimensions,
                },
                transforms_applied: vec![
                    Box::new(Identity {}),
                    Box::new(expected_rotation),
                    Box::new(expected_reflection),
                ],
            };

            let standardized_hex_cell_map = invertible_geometry.compute_standardized_cell_map();

            assert_eq!(
                standardized_snapshot.hex_cell_map,
                standardized_hex_cell_map.standardized_map
            );
            assert_transforms_equal(
                expected_transforms,
                standardized_hex_cell_map.transforms_applied,
            );
        }

        #[test]
        fn computes_a_bounding_polygon_around_hex_0_0() {
            let standardized_snapshot = FourByFour::Standardized.to_snapshot();
            let hex_height = 50.0;
            let hex_widths = HexWidths {
                hex_width: 100.0,
                hex_middle_width: 50.0,
                hex_edge_width: 25.0,
            };
            let expected_polygon = &standardized_snapshot
                .hex_cell_map
                .get(&HexCellCoordinate { row: 0, column: 0 })
                .unwrap()
                .bounding_polygon;

            let bounding_polygon =
                InvertibleStandardizedGeometry::compute_bounding_polygon_around_hex_0_0(
                    hex_height, hex_widths,
                );

            assert_eq!(*expected_polygon, bounding_polygon);
        }
    }

    mod offset_neighbor_coordinates {

        mod add_coordinate_trait {
            use crate::geometry::hexagons::HexCellCoordinate;
            use crate::geometry::hexagons::standardized::{
                OffsetNeighborCoordinate, OffsetNeighborCoordinates,
            };
            use std::collections::HashSet;

            #[test]
            fn offsets_its_coordinates_by_the_added_value() {
                let rhs = HexCellCoordinate { row: 1, column: 2 };
                let maximum = HexCellCoordinate {
                    row: 100,
                    column: 100,
                };
                let coordinates = OffsetNeighborCoordinates {
                    coordinates: vec![
                        OffsetNeighborCoordinate { row: 0, column: 1 },
                        OffsetNeighborCoordinate {
                            row: 80,
                            column: 90,
                        },
                    ],
                    maximum_coordinate: maximum,
                };
                let expected: HashSet<HexCellCoordinate> = HashSet::from([
                    HexCellCoordinate { row: 1, column: 3 },
                    HexCellCoordinate {
                        row: 81,
                        column: 92,
                    },
                ]);

                let added = &coordinates + rhs;

                assert_eq!(expected, added);
            }

            #[test]
            fn removes_coordinates_that_become_invalid() {
                let rhs = HexCellCoordinate {
                    row: 10,
                    column: 20,
                };
                let maximum = HexCellCoordinate {
                    row: 100,
                    column: 100,
                };

                let will_stay_valid = OffsetNeighborCoordinate {
                    row: -1,
                    column: -1,
                };
                let rejected_row_too_small = OffsetNeighborCoordinate {
                    row: -100,
                    column: 10,
                };
                let rejected_column_too_small = OffsetNeighborCoordinate {
                    row: 10,
                    column: -21,
                };
                let rejected_both_too_small = OffsetNeighborCoordinate {
                    row: -11,
                    column: -21,
                };
                let rejected_row_too_large = OffsetNeighborCoordinate {
                    row: 95,
                    column: 10,
                };
                let rejected_column_too_large = OffsetNeighborCoordinate {
                    row: 10,
                    column: 95,
                };
                let rejected_both_too_large = OffsetNeighborCoordinate {
                    row: 99,
                    column: 99,
                };
                let coordinates = OffsetNeighborCoordinates {
                    coordinates: vec![
                        will_stay_valid,
                        rejected_row_too_small,
                        rejected_column_too_small,
                        rejected_both_too_small,
                        rejected_row_too_large,
                        rejected_column_too_large,
                        rejected_both_too_large,
                    ],
                    maximum_coordinate: maximum,
                };
                let expected: HashSet<HexCellCoordinate> =
                    HashSet::from([HexCellCoordinate { row: 9, column: 19 }]);

                let added = &coordinates + rhs;

                assert_eq!(expected, added);
            }
        }
    }

    mod standardized_geometry_definition {
        use crate::PixelPoint;
        use crate::geometry::hexagons::standardized::{
            HexWidths, StandardizedHexGeometryDefinition,
        };
        use crate::geometry::hexagons::test::fixtures::{
            FourByFour, HexGeometrySnapshot, ToSnapshot,
        };

        fn standardized_geometry_from_snapshot(
            snapshot: HexGeometrySnapshot,
        ) -> StandardizedHexGeometryDefinition {
            StandardizedHexGeometryDefinition {
                number_of_rows: snapshot.geometry_definition.number_of_rows,
                number_of_columns: snapshot.geometry_definition.number_of_columns,
                hexagon_height: snapshot.geometry_definition.hexagon_height,
                hexagon_width: snapshot.geometry_definition.hexagon_width,
                geometry_dimensions: snapshot.dimensions,
            }
        }

        #[test]
        fn computes_cell_height() {
            let from_fixture =
                standardized_geometry_from_snapshot(FourByFour::Standardized.to_snapshot());
            let one_more = StandardizedHexGeometryDefinition {
                number_of_rows: 11,
                number_of_columns: 21,
                hexagon_height: 100.0,
                hexagon_width: 300.0,
                geometry_dimensions: PixelPoint { x: 2000, y: 1150 },
            };

            assert_eq!(50.0, from_fixture.compute_hex_height());
            assert_eq!(100.0, one_more.compute_hex_height());
        }

        #[test]
        fn computes_cell_width() {
            let from_fixture =
                standardized_geometry_from_snapshot(FourByFour::Standardized.to_snapshot());
            let one_more = StandardizedHexGeometryDefinition {
                number_of_rows: 11,
                number_of_columns: 21,
                hexagon_height: 100.0,
                hexagon_width: 300.0,
                geometry_dimensions: PixelPoint { x: 5000, y: 1150 },
            };

            let fixture_widths = HexWidths {
                hex_width: 100.0,
                hex_middle_width: 50.0,
                hex_edge_width: 25.0,
            };
            let one_more_widths = HexWidths {
                hex_width: 300.0,
                hex_middle_width: 170.0,
                hex_edge_width: 65.0,
            };

            assert_eq!(
                fixture_widths,
                from_fixture.compute_hex_width(from_fixture.compute_hex_height())
            );
            assert_eq!(
                one_more_widths,
                one_more.compute_hex_width(one_more.compute_hex_height())
            );
        }
    }
}
