use crate::PixelPoint;
use crate::geometry::hexagons::FilledTopLeftCorner::{EMPTY, FILLED};
use crate::geometry::hexagons::FlatSides::FlatVerticalSides;
use crate::geometry::hexagons::transform::{InvertibleStandardizedGeometry, InvertibleTransform};
use crate::geometry::{BoundingPolygon, CellMap, ComputesCellMap};
use serde::Deserialize;
use std::collections::HashMap;
use std::ops::{Add, Mul};

mod transform;

/// A hex map. All rows have the same number of hexes, all columns have the same number of hexes.
///
/// It is possible to input
#[derive(Deserialize, PartialEq, Debug, Clone)]
pub struct HexagonGeometryDefinition {
    flat_sides: FlatSides,
    number_of_rows: u8,
    number_of_columns: u8,
    /// The units here can be pixels, cm, whatever, so long as they're consistent with [hexagon_width]
    hexagon_height: f32,
    /// The units here can be pixels, cm, whatever, so long as they're consistent with [hexagon_height]
    hexagon_width: f32,
    filled_top_left_corner: FilledTopLeftCorner,
}

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
struct StandardizedHexGeometryDefinition {
    number_of_rows: u8,
    number_of_columns: u8,
    hexagon_height: f32,
    hexagon_width: f32,
    geometry_dimensions: PixelPoint,
}

type HexCellMap = HashMap<HexCellCoordinate, HexCell>;
fn offset_map(hex_cell_map: HexCellMap, map_margin: &PixelPoint) -> HexCellMap {
    unimplemented!()
}
fn to_cell_map(hex_cell_map: HexCellMap) -> CellMap {
    unimplemented!()
}

struct StandardizedHexCellMap {
    standardized_map: HexCellMap,
    transforms_applied: Vec<Box<dyn InvertibleTransform>>,
}

impl StandardizedHexCellMap {
    pub(crate) fn invert_standardization(&self) -> HexCellMap {
        todo!()
    }
}

impl ComputesCellMap for HexagonGeometryDefinition {
    fn compute_cell_map(&self, map_dimensions: &PixelPoint, map_margin: &PixelPoint) -> CellMap {
        let map_dimensions_without_margin = PixelPoint {
            x: map_dimensions.x - 2 * map_margin.x,
            y: map_dimensions.y - 2 * map_margin.y,
        };

        let invertible_standardized_geometry =
            InvertibleStandardizedGeometry::standardize(self, map_dimensions_without_margin);
        let standardized_hex_cell_map: StandardizedHexCellMap =
            invertible_standardized_geometry.compute_standardized_cell_map();

        let hex_cell_map = standardized_hex_cell_map.invert_standardization();
        let hex_cell_map = offset_map(hex_cell_map, map_margin);
        to_cell_map(hex_cell_map)
    }
}

impl InvertibleStandardizedGeometry {
    /// Computes a Hex Cell map, using the assumptions that the Hex Geometry is a
    /// [StandardizedHexGeometryDefinition] with no margins.
    fn compute_standardized_cell_map(&self) -> StandardizedHexCellMap {
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

        let bounding_polygon_centered_on_origin =
            Self::compute_bounding_polygon_around_origin(hex_height, hex_width, hex_middle_width);
        let maximum_coordinate = HexCellCoordinate {
            row: self.standardized_geometry.number_of_rows -1,
            column: self.standardized_geometry.number_of_columns-1,
        };
        //TODO: fxn
        let even_column_neighbor_offsets = OffsetNeighborCoordinates {
            coordinates: Vec::from([
                OffsetNeighborCoordinate { row: -1, column: 0 },
                OffsetNeighborCoordinate { row: -1, column: 1},
                OffsetNeighborCoordinate { row: 0, column: 1},
                OffsetNeighborCoordinate { row: 1, column: 0},
                OffsetNeighborCoordinate { row: 0, column: -1},
                OffsetNeighborCoordinate { row: -1, column: -1}

            ]),
            maximum_coordinate
        };
        let odd_column_neighbor_offsets = OffsetNeighborCoordinates {
            coordinates: Vec::from([
                OffsetNeighborCoordinate { row: -1, column: 0 },
                OffsetNeighborCoordinate { row: 0, column: 1},
                OffsetNeighborCoordinate { row: 1, column: 1},
                OffsetNeighborCoordinate { row: 1, column: 0},
                OffsetNeighborCoordinate { row: 0, column: -1},
                OffsetNeighborCoordinate { row: 1, column: -1}
            ]),
            maximum_coordinate
        };

        let mut hex_cell_map: HexCellMap = HashMap::new();
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
                let neighbor_coordinates = if(column_coordinate %2 == 0) {
                    &even_column_neighbor_offsets + hex_coordinate
                } else {
                    &odd_column_neighbor_offsets + hex_coordinate
                };
                let cell = HexCell {
                    center_point,
                    hex_coordinate,
                    neighbor_coordinates,
                    bounding_polygon: bounding_polygon_centered_on_origin
                        .offset_by(center_point)
                        .clamp(self.standardized_geometry.geometry_dimensions),
                };
                hex_cell_map.insert(hex_coordinate, cell);
            }
        }
        StandardizedHexCellMap{
            standardized_map: hex_cell_map,
            transforms_applied: self.transforms_applied, //TODO: UGH
        }
    }

    fn compute_bounding_polygon_around_origin(
        hex_height: f32,
        hex_width: f32,
        hex_middle_width: f32,
    ) -> BoundingPolygon {
        let hex_middle_width_i32 = hex_middle_width as i32;
        let hex_height_i32 = hex_height as i32;
        let hex_width_i32 = hex_width as i32;
        let bounding_polygon_centered_on_origin = BoundingPolygon {
            points: Vec::from([
                PixelPoint {
                    x: hex_middle_width_i32 / 2,
                    y: hex_height_i32,
                },
                PixelPoint {
                    x: hex_width_i32,
                    y: 0,
                },
                PixelPoint {
                    x: hex_middle_width_i32 / 2,
                    y: -hex_height_i32,
                },
                PixelPoint {
                    x: -hex_middle_width_i32 / 2,
                    y: -hex_height_i32,
                },
                PixelPoint {
                    x: -hex_width_i32,
                    y: 0,
                },
                PixelPoint {
                    x: -hex_middle_width_i32,
                    y: hex_height_i32,
                },
            ]),
        };
        bounding_polygon_centered_on_origin
    }
}

struct OffsetNeighborCoordinates {
    coordinates: Vec<OffsetNeighborCoordinate>,
    maximum_coordinate: HexCellCoordinate,
}
#[derive(Debug, Clone, Copy)]
struct OffsetNeighborCoordinate {
    row: i16,
    column: i16,
}
impl Add<HexCellCoordinate> for &OffsetNeighborCoordinates {
    type Output = Vec<HexCellCoordinate>;

    fn add(self, rhs: HexCellCoordinate) -> Vec<HexCellCoordinate> {
        self.coordinates
            .iter()
            .map(|offset| self.addIfValid(offset, rhs))
            .filter(|option| option.is_some())
            .map(|option| option.unwrap())
            .collect()
    }
}

impl OffsetNeighborCoordinates {
    /// Only non-negative coordinates within the allowed [maximum_coordinate] are allowed.
    fn addIfValid(
        &self,
        offset: &OffsetNeighborCoordinate,
        coordinate: HexCellCoordinate,
    ) -> Option<HexCellCoordinate> {
        let row_candidate = coordinate.row as i16 + offset.row;
        let column_candidate = coordinate.column as i16 + offset.column;
        if (row_candidate < 0
            || row_candidate > self.maximum_coordinate.row as i16
            || column_candidate < 0
            || column_candidate > self.maximum_coordinate.column as i16)
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
    /// The first row adds a full cell height, each further row adds just half.
    ///
    /// H = h + (row-1)*h/2.
    ///
    /// Solving for h,
    ///
    /// h = 2H/(1+rows)
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
        (2.0 * total_height) / (self.number_of_rows as f32 + 1.0)
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
        let hex_middle_width = hex_width - hex_edge_width;
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

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
struct HexCellCoordinate {
    row: u8,
    column: u8,
}
impl HexCellCoordinate {
    fn to_coordinate_string(&self) -> String {
        format!("{}-{}", self.column, self.row)
    }
}

#[derive(Debug, PartialEq, Clone)]
struct HexCell {
    hex_coordinate: HexCellCoordinate,
    neighbor_coordinates: Vec<HexCellCoordinate>,
    center_point: PixelPoint,
    bounding_polygon: BoundingPolygon,
}

#[derive(Debug, Clone, Copy)]
struct PositionDelta {
    x: f32,
    y: f32,
}

impl Add<PositionDelta> for PositionDelta {
    type Output = Self;

    fn add(self, rhs: PositionDelta) -> Self {
        PositionDelta {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

impl Mul<u8> for PositionDelta {
    type Output = Self;

    fn mul(self, rhs: u8) -> Self {
        PositionDelta {
            x: self.x * rhs as f32,
            y: self.y * rhs as f32,
        }
    }
}

impl From<PositionDelta> for PixelPoint {
    fn from(value: PositionDelta) -> Self {
        PixelPoint {
            x: value.x.round() as i32,
            y: value.y.round() as i32,
        }
    }
}

#[derive(Deserialize, Debug, PartialEq, Clone, Copy)]
pub enum FlatSides {
    FlatVerticalSides,
    FlatHorizontalSides,
}

/// Whether the top left corner of the map is filled in by a hex.
///
/// This is equivalent to asking if hex (0, 0) exists in the map.
#[derive(Deserialize, Debug, PartialEq, Clone, Copy)]
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
    ///          •     (1,0)    ••••••••••
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
            FlatSides::FlatHorizontalSides => FlatVerticalSides,
            FlatVerticalSides => FlatSides::FlatHorizontalSides,
        }
    }
}
