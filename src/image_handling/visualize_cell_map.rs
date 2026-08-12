use crate::geometry::CellMap;
use ab_glyph::{FontRef, PxScale};
use image::{DynamicImage, ImageFormat, Rgba};
use imageproc::drawing::{draw_cross_mut, draw_text_mut};
use std::path::PathBuf;

/// Draws a [CellMap] onto a copy of the original_iamge, saving the result. This allows
/// a visual inspection of what was generated.
pub fn save_cell_map_visualization(
    target_directory: &PathBuf,
    cell_map: &CellMap,
    original_image: &DynamicImage,
    visualization_color: &Rgba<u8>,
) {
    let mut visualization_image = original_image.clone();
    let text_scale = PxScale { x: 10.0, y: 10.0 };
    let font = FontRef::try_from_slice(include_bytes!("RobotoMono-Regular.ttf")).unwrap();
    cell_map
        .cells_by_coordinate
        .iter()
        .for_each(|(coordinate, cell)| {
            cell.bounding_polygon
                .draw(&mut visualization_image, *visualization_color);
            draw_cross_mut(
                &mut visualization_image,
                *visualization_color,
                cell.center_point.x,
                cell.center_point.y,
            );
            draw_text_mut(
                &mut visualization_image,
                *visualization_color,
                cell.center_point.x,
                cell.center_point.y,
                text_scale,
                &font,
                coordinate,
            )
        });
    let mut path = PathBuf::new();
    path.push(target_directory);
    path.push("cell_map_visualization.png");
    visualization_image
        .save_with_format(&path, ImageFormat::Png)
        .unwrap();
}

#[cfg(test)]
mod test {
    use crate::geometry::hexagons::fixtures::{FourByFour, ToSnapshot};
    use crate::image_handling::test::fixtures::FourByFourImages;
    use crate::image_handling::visualize_cell_map::save_cell_map_visualization;
    use image::Rgba;

    #[test]
    fn draws_a_cell_maps_borders_centers_and_coordinate() {
        let original_map_image = FourByFourImages::Standardized.load_image();
        let snapshot = FourByFour::Standardized.to_snapshot();
        let mut expected_path = FourByFourImages::Standardized.get_test_cases_path();
        expected_path.push("visualize_cell_map/expected.png");
        let expected_image = image::ImageReader::open(expected_path)
            .unwrap()
            .decode()
            .unwrap();

        let mut target_directory = FourByFourImages::Standardized.get_test_cases_path();
        target_directory.push("visualize_cell_map/result");
        let mut target_file = target_directory.clone();
        target_file.push("cell_map_visualization.png");

        let red = Rgba::from([255u8, 0, 0, 255]);
        save_cell_map_visualization(
            &target_directory,
            &snapshot.cell_map,
            &original_map_image,
            &red,
        );
        let saved_image = image::open(target_file).unwrap();

        assert_eq!(expected_image, saved_image)
    }
}
