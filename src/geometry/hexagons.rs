use crate::PixelPoint;
use crate::geometry::hexagons::FilledTopLeftCorner::{EMPTY, FILLED};
use crate::geometry::hexagons::FlatSides::FlatVerticalSides;
use crate::geometry::{BoundingPolygon, CellMap, ComputesCellMap};
use serde::Deserialize;
use std::collections::HashMap;
use crate::geometry::hexagons::transform::{InvertibleStandardizedGeometry, InvertibleTransform};

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
    geometry_dimensions: PixelPoint
}

type HexCellMap = HashMap<HexCellCoordinate, HexCell>;
fn to_cell_map(hex_cell_map: HexCellMap) -> CellMap {
    unimplemented!()
}
struct StandardizedHexCellMap {
    standardized_map: HexCellMap,
    transforms_applied: Vec<Box<dyn InvertibleTransform>>
}

impl ComputesCellMap for HexagonGeometryDefinition {
    
    fn compute_cell_map(&self, map_dimensions: &PixelPoint, map_margin: &PixelPoint) -> CellMap {
        let map_dimensions_without_margin = PixelPoint {
            x: map_dimensions.x - 2 * map_margin.x,
            y: map_dimensions.y - 2 * map_margin.y,
        };

        let invertible_standardized_geometry = InvertibleStandardizedGeometry::standardize(self, map_dimensions_without_margin);
        let (standardized_geometry, transforms_applied) = (invertible_standardized_geometry.standardized_geometry, invertible_standardized_geometry.transforms_applied);
        let standardized_hex_cell_map: StandardizedHexCellMap = standardized_geometry.compute_standardized_cell_map();
        
        let hex_cell_map = standardized_hex_cell_map.invert_standardization(transforms_applied);
        let hex_cell_map = self.offset_map(hex_cell_map, map_margin);
        to_cell_map(hex_cell_map);
    }
}

impl InvertibleStandardizedGeometry {
    
}

impl StandardizedHexGeometryDefinition {

    /// Computes a Hex Cell map, using the assumptions that the Hex Geometry is a
    /// [StandardizedHexGeometryDefinition] with no margins.
    fn compute_standardized_cell_map(&self) -> StandardizedHexCellMap {
        let hex_height = self.compute_hex_height();
        let hex_widths = self.compute_hex_width(hex_height);
        let (hex_width, hex_middle_width, hex_edge_with) = (hex_widths.hex_width, hex_widths.hex_middle_width, hex_widths.hex_edge_width);

        let hex_0_0_center = PositionDelta{x: hex_width/2.0, y: hex_height/2.0};
        let row_position_delta = PositionDelta {x: 0.0, y: hex_height};
        let column_position_delta = PositionDelta{x: hex_middle_width + hex_edge_with, y:0.0};
        let odd_column_offset = PositionDelta{x: 0.0, y: hex_height/2.0};
        
        
        
        for row_coordinate in 0..self.number_of_rows {
            for column_coordinate in 0.. self.number_of_columns {
                let coordinate = HexCellCoordinate{row: row_coordinate, column: column_coordinate};
                
            }
        }

        todo!()
    }

    fn offset_map(&self, p0: _, p1: &PixelPoint) -> _ {
        todo!()
    }

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
        let hex_edge_width = (self.geometry_dimensions.x as f32- self.number_of_columns as f32 * hex_width)/(1.0-self.number_of_columns as f32);
        let hex_middle_width = hex_width - hex_edge_width;
        HexWidths{
            hex_width,
            hex_middle_width,
            hex_edge_width
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
    hex_edge_width: f32
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

struct PositionDelta {
    x: f32,
    y: f32,
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
