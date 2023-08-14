use ansi_term::Colour::RGB;
use image::codecs::gif::GifDecoder;
use image::{imageops::FilterType, io::Reader as ImageReader};
use image::{AnimationDecoder, DynamicImage, RgbaImage};
use ndarray::{Array2, Axis};
use std::fs::File;
use std::path::Path;
use std::str::FromStr;
use std::time::Duration;
use std::{env, thread};

fn main() {
    let args = env::args().collect::<Vec<String>>();

    if args.len() < 2 {
        println!("{}", get_help());
        return;
    }

    let size = termion::terminal_size().unwrap();

    let mut config = Config::from_vec(&args);
    config.term_size = size;

    let extension = Path::new(&config.path).extension();
    match extension {
        Some(p) if p == "gif" => show_animation(&config),
        _ => show_image(&config),
    }
}

fn show_animation(config: &Config) {
    match File::open(&config.path) {
        Ok(img_file) => match GifDecoder::new(img_file) {
            Ok(decoder) => animate(decoder, config),
            Err(err) => println!("Unable to decode GIF image: {}", err),
        },
        Err(err) => println!("Unable to open file {}: {}", config.path, err),
    }
}

fn animate(decoder: GifDecoder<File>, config: &Config) {
    let frames = decoder.into_frames().collect_frames().unwrap();
    for frame in frames {
        let (numer, denom) = frame.delay().numer_denom_ms();
        let micros = 1000 * numer / denom;

        let image = DynamicImage::from(RgbaImage::from(frame.into_buffer()));
        let img_mat = image_to_color_matrix(image, config);
        print_color_image_ansi(img_mat);

        let duration = Duration::from_micros(micros as u64);
        thread::sleep(duration);
    }
}

fn show_image(config: &Config) {
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

fn image_to_color_matrix(image: DynamicImage, config: &Config) -> Array2<[u8; 3]> {
    let (width, height) = config.term_size;
    let width = width as u32;
    let height = 2 * height as u32;

    let filter = FilterType::CatmullRom;
    let resized_img = match config.resize_type {
        ResizeType::Fit => image.resize(width, height, filter),
        ResizeType::CropToFill => image.resize_to_fill(width, height, filter),
        ResizeType::ScaleToFill => image.resize_exact(width, height, filter),
    };

    let width_new = resized_img.width();
    let height_new = resized_img.height();
    let color_img = match config.image_type {
        ImageType::Color => resized_img
            .into_rgb8()
            .enumerate_pixels()
            .map(|(_, _, rgb)| rgb.0)
            .collect::<Vec<[u8; 3]>>(),
        ImageType::Gray => resized_img
            .into_luma8()
            .enumerate_pixels()
            .map(|(_, _, p)| [p.0[0], p.0[0], p.0[0]])
            .collect::<Vec<[u8; 3]>>(),
    };
    let matrix = Array2::from_shape_vec((height_new as usize, width_new as usize), color_img);

    matrix.unwrap()
}

fn print_color_image_ansi(image: Array2<[u8; 3]>) {
    let mut text: String = String::new();
    for i in 0..image.len_of(Axis(0)) / 2 {
        for j in 0..image.len_of(Axis(1)) {
            let px_upper = image[[2 * i, j]];
            let px_lower = image[[2 * i + 1, j]];
            let st = RGB(px_lower[0], px_lower[1], px_lower[2])
                .on(RGB(px_upper[0], px_upper[1], px_upper[2]))
                .paint("▄")
                .to_string();
            text.push_str(st.as_str());
        }
        text.push('\n');
    }
    println!("{}", text);
}

fn get_help() -> String {
    "Terminal image viewer (tiv)\n\
     ---------------------------\n\
     tiv-rs <path_to_image> (resize_type (image_type))\n\
     \tresize type ... f (fit; default) | c (crop to fill) | s (scale to fill)\n\
     \timage type  ... C (color; default) | G (gray))"
        .to_string()
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
            1 => panic!("{}", get_help()),
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

#[derive(Debug, Default)]
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