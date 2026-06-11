use image::DynamicImage;
use std::path::PathBuf;

pub enum FourByFourImages {
    Standardized,
}

impl FourByFourImages {
    pub fn load_image(&self) -> DynamicImage {
        match self {
            FourByFourImages::Standardized => {
                let mut path = self.get_fixture_path();
                path.push("map_images/standard_four_by_four.png");
                image::open(path).unwrap()
            }
        }
    }

    pub fn get_test_resources_path() -> PathBuf {
        let mut path = PathBuf::new();
        path.push(env!("CARGO_MANIFEST_DIR"));
        path.push("test_resources");
        path
    }

    pub fn get_fixture_path(&self) -> PathBuf {
        let mut path = Self::get_test_resources_path();
        match self {
            FourByFourImages::Standardized => {
                path.push("four_by_four_hex_map");
                path
            }
        }
    }

    pub fn get_test_cases_path(&self) -> PathBuf {
        match self {
            FourByFourImages::Standardized => {
                let mut path = self.get_fixture_path();
                path.push("test_cases");
                path
            }
        }
    }
}
