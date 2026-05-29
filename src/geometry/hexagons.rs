use crate::PixelPoint;
use crate::geometry::hexagons::FilledTopLeftCorner::{EMPTY, FILLED};
use crate::geometry::hexagons::FlatSides::FlatVerticalSides;
use crate::geometry::{Cell, CellMap, ComputesCellMap};
use serde::Deserialize;
use std::collections::HashMap;
use std::iter::Copied;

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
    bounding_polygon: Vec<PixelPoint>,
}

type HexCellMap = HashMap<HexCellCoordinate, HexCell>;

//TODO: Actually, do this computation ONCE for the flat filled version, rotate the others into this position. Much easier than a thousand ifs
impl ComputesCellMap for HexagonGeometryDefinition {
    fn compute_cell_map(&self, map_dimensions: &PixelPoint, map_margin: &PixelPoint) -> CellMap {
        let map_dimensions_without_margin = PixelPoint {
            x: map_dimensions.x - 2 * map_margin.x,
            y: map_dimensions.y - 2 * map_margin.y,
        };
        let (map_width, map_height) = (
            map_dimensions_without_margin.x as f32,
            map_dimensions_without_margin.y as f32,
        );
        let hex_width_height = self.determine_hex_dimensions((map_width, map_height));
        let (hex_width, hex_height) = (hex_width_height.width, hex_width_height.height);

        //TODO: This lets us get a precise measure of the angle, actually
        // The distance a change in the row or column coordinate moves us from the position of hex (0,0).
        let (rowPositionDelta, columnPositionDelta, offsetDelta) = match self.flat_sides {
            FlatSides::FlatVerticalSides => (
                PositionDelta {
                    x: (map_height - hex_height) / (self.number_of_rows as f32 - 1.0),
                    y: 0.0,
                },
                PositionDelta {
                    x: hex_width,
                    y: 0.0,
                },
                PositionDelta {
                    x: hex_width / 2.0,
                    y: 0.0,
                },
            ),
            FlatSides::FlatHorizontalSides => (
                PositionDelta {
                    x: 0.0,
                    y: hex_height,
                },
                PositionDelta {
                    x: (map_width - hex_width) / (self.number_of_columns as f32 - 1.0),
                    y: 0.0,
                },
                PositionDelta {
                    x: 0.0,
                    y: hex_height / 2.0,
                },
            ),
        };

        todo!()
        //flat: number of cells, length. be it x or y, rows or cols.
        //non-flat:
    }
}

impl HexagonGeometryDefinition {
    /// The dimension in the direction of flatness can be easily computed, see [compute_flat_cell_dimension].
    ///
    /// The other dimension is derived from the ratio that user inputs.
    fn determine_hex_dimensions(&self, (map_width, map_height): (f32, f32)) -> WidthHeight {
        match self.flat_sides {
            FlatSides::FlatVerticalSides => {
                let width = compute_flat_cell_dimension(map_width, self.number_of_columns as f32);
                WidthHeight {
                    height: width * self.hexagon_height / self.hexagon_width,
                    width,
                }
            }
            FlatSides::FlatHorizontalSides => {
                let height = compute_flat_cell_dimension(map_height, self.number_of_rows as f32);
                WidthHeight {
                    width: height * self.hexagon_width / self.hexagon_height,
                    height,
                }
            }
        }
    }
}

struct PositionDelta {
    x: f32,
    y: f32,
}

struct WidthHeight {
    height: f32,
    width: f32,
}

/// The "flat" cell dimension is the the size of the cell in the direction of flatness.
///
/// E.g. if we have [FlatHorizontalSides], then it is the height of the cell.
///
/// The first row adds a full cell height, each further row adds just half.
///
/// H = h + (row-1)*h/2.
///
/// Solving for h,
///
/// h = 2H/(1+rows)
///
///``` picture
///                      ┌─────╴     ••••••••••                 ╶──────┐
///                      │          •          •                       │
///                      │         •            •                      │
///      h - cell height │        •              ••••••••••            │
///                      │         •            •          •           │
///                      │          •          •            •          │
///                      └─────╴     ••••••••••              •         │
///                                 •          •            •          │  H - total height
///                                •            •          •           │
///                               •              ••••••••••            │
///                                •            •                      │
///                                 •          •                       │
///                                  ••••••••••                 ╶──────┘
///```
///
fn compute_flat_cell_dimension(total_flat_cell_dimension: f32, number_of_flat_cells: f32) -> f32 {
    (2.0 * total_flat_cell_dimension) / (number_of_flat_cells + 1.0)
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
