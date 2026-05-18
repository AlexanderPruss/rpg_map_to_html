use serde::Deserialize;
use crate::geometry::{CellMap, ComputesCellMap};

#[derive(Deserialize, Debug)]
pub struct HexagonGeometryDefinition {
    flat_sides: FlatSides,
    number_of_rows: i8,
    number_of_columns: i8,
    /// The units here can be pixels, cm, whatever, so long as they're consistent with [hexagon_width]
    hexagon_height: f32,
    /// The units here can be pixels, cm, whatever, so long as they're consistent with [hexagon_height]
    hexagon_width: f32,
    filled_top_left_corner: FilledTopLeftCorner,
    row_details: Option<HexRowColumnDetails>,
    column_details: Option<HexRowColumnDetails>
}

impl ComputesCellMap for HexagonGeometryDefinition {
    fn compute_cell_map<'map>(&self) -> CellMap<'map> {
        todo!()
    }
}

#[derive(Deserialize, Debug)]
pub enum FlatSides {
    FlatVerticalSides,
    FlatHorizontalSides
}

/// Detailed information about the layout of Rows or Columns.
///
/// Currently only uniform hex maps are supported.
#[derive(Deserialize, Debug)]
pub enum HexRowColumnDetails {
    ///All rows or columns have the same number of hexes.
    UNIFORM
}

/// Whether the top left corner of the map is filled in by a hex.
#[derive(Deserialize, Debug)]
pub enum FilledTopLeftCorner {
    /// There's an empty hex at the top left of the map. For flat top hexes, the corner looks like this:
    ///
    ///                         ••••••••••
    ///                        •          •
    ///                       •            •
    ///             ••••••••••              •
    ///            •          •            •
    ///           •            •          •
    ///          •              ••••••••••
    ///           •            •
    ///            •          •
    ///             ••••••••••
    ///
    EMPTY,
    /// There's no empty space at the top left of the map. For flat top hexes, the corner looks like this:
    ///
    ///                        ••••••••••
    ///                       •          •
    ///                      •            •
    ///                     •              ••••••••••
    ///                      •            •          •
    ///                       •          •            •
    ///                        ••••••••••              •
    ///                                  •            •
    ///                                   •          •
    ///                                    ••••••••••
    FILLED
}
