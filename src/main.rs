use ansi_term::Colour::RGB;
use image::DynamicImage;
use image::{imageops::FilterType, io::Reader as ImageReader};
use ndarray::{Array2, Axis};
use std::str::FromStr;
use std::env;

fn main() {
    let args = env::args().collect::<Vec<String>>();

    let size = termion::terminal_size().unwrap();

    let mut config = Config::from_vec(&args);
    config.term_size = size;

    let image = read_image(&config.path);
    match image {
        Ok(img) => {
            let img_mat = image_to_color_matrix(img, config);
            print_color_image_ansi(img_mat);
        }
        Err(err) => println!("Path to image '{}': {}", &config.path, err),
    }
}

fn read_image(path: &str) -> Result<DynamicImage, image::ImageError> {
    let image = ImageReader::open(path)?.decode();

    image
}

fn image_to_color_matrix(image: DynamicImage, config: Config) -> Array2<[u8;3]> {
    let (width, height) = config.term_size;
    let width = width as u32;
    let height = height as u32;
    
    let filter = FilterType::CatmullRom;
    let resized_img = match config.resize_type {
        ResizeType::Fit => image.resize(width, height, filter),
        ResizeType::CropToFill => image.resize_to_fill(width, height, filter),
        ResizeType::ScaleToFill => image.resize_exact(width, height, filter),
    };

    let width_new = resized_img.width();
    let height_new = resized_img.height();
    let color_img = match config.image_type {
        ImageType::Color => resized_img.into_rgb8().enumerate_pixels().map(|(_,_,rgb)| rgb.0 ).collect::<Vec<[u8;3]>>(),
        ImageType::Gray => resized_img.into_luma8().enumerate_pixels().map(|(_,_,p)| [p.0[0],p.0[0],p.0[0]]).collect::<Vec<[u8;3]>>(),
    };
    let matrix = Array2::from_shape_vec((height_new as usize, width_new as usize), color_img);

    matrix.unwrap()
}

fn print_color_image_ansi(image: Array2<[u8;3]>) {
    let mut text: String = String::new();
    for row in image.axis_iter(Axis(0)) {
        for px in row.iter() {
            let st = RGB(px[0],px[1],px[2]).paint("█").to_string();
            text.push_str(st.as_str());
        }
        text.push('\n');
    }
    println!("{}", text);
}

#[derive(Debug, Default)]
struct Config {
    path: String,
    resize_type: ResizeType,
    term_size: (u16, u16),
    image_type: ImageType,
}

impl Config {
    fn from_vec(value: &Vec<String>) -> Self {
        match value.len() {
            1 => panic!("Wrong input parameters. Expects: <path_to_image> (f|c|s) (C|G)"),
            2 => Config {
                path: value[1].clone(),
                ..Default::default()
            },
            3 => Config {
                path: value[1].clone(),
                resize_type: ResizeType::from_str(value[2].as_str()).unwrap(),
                image_type: Default::default(),
                term_size: Default::default(),
            },
            _ => Config {
                path: value[1].clone(),
                resize_type: ResizeType::from_str(value[2].as_str()).unwrap(),
                image_type: ImageType::from_str(value[3].as_str()).unwrap(),
                term_size: Default::default(),
            },
        }
    }
}

#[derive(Debug, Default)]
enum ResizeType {
    #[default]
    Fit, // preserve aspect ratio
    CropToFill,  // preserve aspect ratio
    ScaleToFill, // changes aspect ratio
}

impl FromStr for ResizeType {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "f" => Ok(ResizeType::Fit),
            "c" => Ok(ResizeType::CropToFill),
            "s" => Ok(ResizeType::ScaleToFill),
            _ => Err(()),
        }
    }
}

#[derive(Debug,Default)]
enum ImageType {
    #[default]
    Color,
    Gray,
}

impl FromStr for ImageType {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "C" => Ok(ImageType::Color),
            "G" => Ok(ImageType::Gray),
            _ => Err(()),
        }
    }
}