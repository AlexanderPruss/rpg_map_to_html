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
                let neighbor_coordinates = if column_coordinate % 2 == 0  {
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

pub(super) struct OffsetNeighborCoordinates {
    pub coordinates: Vec<OffsetNeighborCoordinate>,
    pub maximum_coordinate: HexCellCoordinate,
}
#[derive(Debug, Clone, Copy)]
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
struct HexWidths {
    hex_width: f32,
    hex_middle_width: f32,
    hex_edge_width: f32,
}

#[cfg(test)]
mod test {
    use super::*;

    mod standardized_hex_cell_map {
        #[test]
        fn inverts_its_hex_map_by_reversing_the_applied_transforms_in_reverse_order() {
            unimplemented!()
        }
    }

    mod invertible_standardized_geometry {
        #[test]
        fn computes_cell_maps_for_standardized_geometries() {
            unimplemented!()
        }

        #[test]
        fn computes_a_bounding_polygon_around_hex_0_0() {
            unimplemented!()
        }
    }

    mod offset_neighbor_coordinates {

        mod add_coordinate_trait {
            #[test]
            fn offsets_its_coordinates_by_the_added_value() {
                unimplemented!()
            }

            #[test]
            fn removes_coordinates_that_become_invalid() {
                unimplemented!()
            }
        }

        mod offset_coordinate_if_valid {
            #[test]
            fn offsets_the_coordinate() {
                unimplemented!()
            }

            #[test]
            fn rejects_the_coordinate_if_x_becomes_negative() {
                unimplemented!()
            }

            #[test]
            fn rejects_the_coordinate_if_y_becomes_negative() {
                unimplemented!()
            }

            #[test]
            fn rejects_the_coordinate_if_x_becomes_too_large() {
                unimplemented!()
            }

            #[test]
            fn rejects_the_coordinate_if_y_becomes_too_large() {
                unimplemented!()
            }
        }
    }

    mod standardized_geometry_definition {

        #[test]
        fn computes_cell_height() {
            unimplemented!()
        }

        #[test]
        fn computes_cell_width() {
            unimplemented!()
        }
    }
}
