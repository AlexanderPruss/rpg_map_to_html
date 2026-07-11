use crate::PixelPoint;
use crate::geometry::BoundingPolygon;
use crate::image_handling::map_cutout::CutoutImage;
use crate::image_handling::table_of_contents::TableOfContentsMapImage;
use image::{DynamicImage, Rgba};
use imageproc::drawing::draw_line_segment_mut;
use std::fs::File;
use std::io::{BufReader, Write};
use std::ops::Add;
use std::path::PathBuf;

pub mod image_config;

pub mod map_cutout;
pub mod table_of_contents;
pub mod visualize_cell_map;

mod empty_cell_detection;

pub static IMAGE_SUBDIRECTORY: &str = "generated-images";
pub static CACHED_TABLE_OF_CONTENTS_FILE: &str = "table-of-contents-metadata.json";
pub static CACHED_CUTOUT_IMAGES_FILE: &str = "cutout-images-metadata.json";

/// Saves a cell map. Used primarily for caching.
pub fn persist_image_metadata(
    target_directory: &PathBuf,
    image_filename: &String,
    image: &DynamicImage,
    table_of_contents_images: &Vec<TableOfContentsMapImage>,
    cutout_images: &Vec<CutoutImage>,
) {
    let serialized_table_of_contents =
        serde_json::to_string_pretty(table_of_contents_images).unwrap();
    let mut path = PathBuf::from(target_directory);
    path.push(CACHED_TABLE_OF_CONTENTS_FILE);
    let mut table_of_contents_file = File::create(&path).unwrap();
    table_of_contents_file
        .write(serialized_table_of_contents.as_bytes())
        .unwrap();
    table_of_contents_file.flush().unwrap();

    let serialized_cutout_images = serde_json::to_string_pretty(cutout_images).unwrap();
    let mut path = PathBuf::from(target_directory);
    path.push(CACHED_CUTOUT_IMAGES_FILE);
    let mut cutout_images_file = File::create(&path).unwrap();
    cutout_images_file
        .write(serialized_cutout_images.as_bytes())
        .unwrap();
    cutout_images_file.flush().unwrap();

    let mut cached_image_path = PathBuf::from(target_directory);
    cached_image_path.push(image_filename);
    image.save(cached_image_path).unwrap();
}

/// Loads the cell map persisted by [persist_cell_map_as_geometry]. Returns NONE if the file can't
/// be found or parsed.
pub fn load_persisted_image_metadata(
    target_directory: &PathBuf,
    image_filename: &String,
) -> Option<(Vec<TableOfContentsMapImage>, Vec<CutoutImage>, DynamicImage)> {
    let mut path = PathBuf::from(target_directory);
    path.push(CACHED_TABLE_OF_CONTENTS_FILE);
    let table_of_contents_file = File::open(path);
    if table_of_contents_file.is_err() {
        return None;
    };
    let table_of_contents_result: serde_json::Result<Vec<TableOfContentsMapImage>> =
        serde_json::from_reader(BufReader::new(table_of_contents_file.unwrap()));
    if table_of_contents_result.is_err() {
        return None;
    }

    let mut path = PathBuf::from(target_directory);
    path.push(CACHED_CUTOUT_IMAGES_FILE);
    let cutout_images_file = File::open(path);
    if cutout_images_file.is_err() {
        return None;
    };
    let cutout_images_result: serde_json::Result<Vec<CutoutImage>> =
        serde_json::from_reader(BufReader::new(cutout_images_file.unwrap()));
    if cutout_images_result.is_err() {
        return None;
    }

    let mut path = PathBuf::from(target_directory);
    path.push(image_filename);
    let image = image::open(path);
    if image.is_err() {
        return None;
    }
    Some((
        table_of_contents_result.unwrap(),
        cutout_images_result.unwrap(),
        image.unwrap(),
    ))
}

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
    use crate::image_handling::test::fixtures::get_test_resources_path;
    use image::DynamicImage;

    pub(crate) mod fixtures;

    pub fn _save_to_out(image: &DynamicImage) {
        let mut output_path = get_test_resources_path();
        output_path.push("out.png");
        image.save(output_path).unwrap()
    }

    mod persist_and_load_computed_images {
        use crate::PixelPoint;
        use crate::image_handling::map_cutout::CutoutImage;
        use crate::image_handling::table_of_contents::TableOfContentsMapImage;
        use crate::image_handling::test::fixtures::FourByFourImages;
        use crate::image_handling::{load_persisted_image_metadata, persist_image_metadata};
        use std::collections::HashSet;
        use std::path::PathBuf;

        #[test]
        fn persists_and_loads_computed_images() {
            let mut test_case_path = PathBuf::new();
            test_case_path.push(env!("CARGO_MANIFEST_DIR"));
            test_case_path.push("test_resources");
            test_case_path.push("persist_image_metadata");

            let mut target_directory = PathBuf::from(&test_case_path);
            target_directory.push("result");
            let table_of_contents_images: Vec<TableOfContentsMapImage> = vec![
                TableOfContentsMapImage {
                    filename: "first_toc.jpg".to_string(),
                    size: PixelPoint { x: 1, y: 2 },
                    offset: PixelPoint { x: 3, y: 4 },
                    coordinates_contained: HashSet::from(["foo".to_string(), "bar".to_string()]),
                },
                TableOfContentsMapImage {
                    filename: "second_toc.jpg".to_string(),
                    size: PixelPoint { x: 4, y: 5 },
                    offset: PixelPoint { x: 6, y: 7 },
                    coordinates_contained: HashSet::from(["baz".to_string(), "dag".to_string()]),
                },
            ];
            let cutout_images: Vec<CutoutImage> = vec![
                CutoutImage {
                    coordinate: "first_cutout.png".to_string(),
                    offset_from_original_image: PixelPoint { x: 10, y: 20 },
                    image_size: PixelPoint { x: 30, y: 40 },
                },
                CutoutImage {
                    coordinate: "second_cutout.png".to_string(),
                    offset_from_original_image: PixelPoint { x: 100, y: 200 },
                    image_size: PixelPoint { x: 300, y: 400 },
                },
            ];
            let filename = "four-by-four.png".to_string();
            let image = FourByFourImages::Standardized.load_image();
            persist_image_metadata(
                &target_directory,
                &filename,
                &image,
                &table_of_contents_images,
                &cutout_images,
            );
            let loaded_metadata =
                load_persisted_image_metadata(&target_directory, &filename).unwrap();
            assert_eq!(table_of_contents_images, loaded_metadata.0);
            assert_eq!(cutout_images, loaded_metadata.1);
            assert_eq!(image, loaded_metadata.2);
        }
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
