use crate::PixelPoint;
use crate::geometry::BoundingPolygon;
use image::{DynamicImage, Rgba};
use imageproc::drawing::draw_line_segment_mut;
use std::ops::Add;

pub mod map_cutout;
pub mod table_of_contents;
pub mod visualize_cell_map;

mod empty_cell_detection;
mod image_config;

impl BoundingPolygon {
    /// Draws the polygon's line segments onto the given [image] in-place.
    /// Lines are drawn with a + shaped brush.
    fn draw(&self, image: &mut DynamicImage, line_color: Rgba<u8>) {
        if self.points.len() == 0 {
            return;
        }

        //The pixel, but also the pixels to the left, right, top, and bottom.
        let brush_offsets = vec![
            PixelPoint { x: 0, y: 0 },
            PixelPoint { x: 1, y: 0 },
            PixelPoint { x: -1, y: 0 },
            PixelPoint { x: 0, y: 1 },
            PixelPoint { x: 0, y: -1 },
        ];
        let mut previous_point = self.points.last().unwrap();
        self.points.iter().for_each(|pixel_point| {
            brush_offsets.iter().for_each(|brush| {
                let previous_brush_point = previous_point + brush;
                let brush_point = pixel_point + brush;
                draw_line_segment_mut(
                    image,
                    (previous_brush_point.x as f32, previous_brush_point.y as f32),
                    (brush_point.x as f32, brush_point.y as f32),
                    line_color,
                );
            });
            previous_point = pixel_point;
        });
    }
}

impl Add<&PixelPoint> for &PixelPoint {
    type Output = PixelPoint;

    fn add(self, rhs: &PixelPoint) -> Self::Output {
        PixelPoint {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

#[cfg(test)]
mod test {
    use crate::image_handling::test::fixtures::{FourByFourImages, get_test_resources_path};
    use image::DynamicImage;

    pub(crate) mod fixtures;

    pub fn _save_to_out(image: &DynamicImage) {
        let mut output_path = get_test_resources_path();
        output_path.push("out.png");
        image.save(output_path).unwrap()
    }

    mod draw_bounding_polygon {
        use crate::geometry::BoundingPolygon;
        use crate::geometry::hexagons::fixtures::{FourByFour, ToSnapshot};
        use crate::image_handling::test::fixtures::FourByFourImages;
        use image::Rgba;

        #[test]
        fn draws_all_polygon_line_segments() {
            let mut image_with_polygon = FourByFourImages::Standardized.load_image();
            let snapshot = FourByFour::Standardized.to_snapshot();
            let mut expected_path = FourByFourImages::Standardized.get_test_cases_path();
            expected_path.push("draw_polygon/draw_1_0_polygon/expected.png");
            let expected_image = image::ImageReader::open(expected_path)
                .unwrap()
                .decode()
                .unwrap();

            let cell_001_000 = snapshot
                .cell_map
                .cells_by_coordinate
                .get(&String::from("001.000"))
                .unwrap();
            let red = Rgba::from([255u8, 0, 0, 255]);
            cell_001_000
                .bounding_polygon
                .draw(&mut image_with_polygon, red);

            assert_eq!(expected_image, image_with_polygon)
        }

        #[test]
        fn draws_all_polygon_line_segments_for_hexes_on_the_border() {
            let mut image_with_polygon = FourByFourImages::Standardized.load_image();
            let snapshot = FourByFour::Standardized.to_snapshot();
            let mut expected_path = FourByFourImages::Standardized.get_test_cases_path();
            expected_path.push("draw_polygon/draw_0_0_polygon/expected.png");

            let expected_image = image::ImageReader::open(expected_path)
                .unwrap()
                .decode()
                .unwrap();

            let cell_000_000 = snapshot
                .cell_map
                .cells_by_coordinate
                .get(&String::from("000.000"))
                .unwrap();
            let red = Rgba::from([255u8, 0, 0, 255]);
            cell_000_000
                .bounding_polygon
                .draw(&mut image_with_polygon, red);

            assert_eq!(expected_image, image_with_polygon)
        }

        #[test]
        fn does_nothing_if_the_polygon_has_no_points() {
            let mut image_with_polygon = FourByFourImages::Standardized.load_image();
            let expected_image = image_with_polygon.clone();
            let empty_polygon = BoundingPolygon { points: vec![] };

            let red = Rgba::from([255u8, 0, 0, 255]);
            empty_polygon.draw(&mut image_with_polygon, red);

            assert_eq!(expected_image, image_with_polygon)
        }
    }
}
