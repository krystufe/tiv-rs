use image::DynamicImage;
use image::{imageops::FilterType, io::Reader as ImageReader, GrayImage, Luma};
use ndarray::{Array2, Axis};
use std::{env, default};
use ansi_term::Colour::RGB;

fn main() {
    let args = env::args().collect::<Vec<String>>();

    let path = get_path(&args);
    println!("Path ... {}", path);

    let size = termion::terminal_size().unwrap();

    let config = Config{ path: path.to_string(), term_size: size, ..Default::default() };

    let image = read_image(path);
    match image {
        Ok(img) => {
            let img_mat = image_to_matrix(img, size);
            //print_image(img_mat, &palette_5);
            //print_image_ansi(img_mat, &ansi_8biy_gray);
            print_image_ansi(img_mat, &ansi_gray);
        }
        Err(err) => println!("Path to image '{}': {}", path, err),
    }
}

fn get_path(args: &Vec<String>) -> &str {
    args[1].as_str()
}

fn read_image(path: &str) -> Result<DynamicImage, image::ImageError> {
    let image = ImageReader::open(path)?.decode();

    image
}

fn image_to_matrix(image: DynamicImage, size: (u16, u16)) -> Array2<f64> {
    let (width, height) = size;
    let resized_img =  image
        .resize_exact(width as u32, height as u32, FilterType::CatmullRom);
    let width_new = resized_img.width();
    let height_new = resized_img.height();
    let gray_img = resized_img
        .into_luma8()
        .iter()
        .map(|p| (*p as f64) / 255.0)
        .collect::<Vec<f64>>();
    let matrix = Array2::from_shape_vec((height_new as usize,width_new as usize), gray_img);

    matrix.unwrap()
}

fn print_image(image: Array2<f64>, palette: &dyn Fn(&f64) -> char) {
    let mut text: String = String::new();
    for row in image.axis_iter(Axis(0)) {
        for value in row.iter() {
            let ch = palette(value);
            text.push(ch);
        }
        text.push('\n');
    }
    println!("{}", text);
}

fn print_image_ansi(image: Array2<f64>, palette: &dyn Fn(&f64) -> String) {
    let mut text: String = String::new();
    for row in image.axis_iter(Axis(0)) {
        for value in row.iter() {
            let st = palette(value);
            text.push_str(st.as_str());
        }
        text.push('\n');
    }
    println!("{}", text);
}

fn ansi_gray(value: &f64) -> String{
    let val = (255.0 * value) as u8; 
    RGB(val,val,val).paint("█").to_string()
}

fn palette_5(value: &f64) -> char{
    match value {
        v if *v < 0.2 => ' ',
        v if *v < 0.4 => '░',
        v if *v < 0.6 => '▒',
        v if *v < 0.8 => '▓',
        _ => '█',
    }
}

fn palette_7(value: &f64) -> char{
    match value {
        v if *v < 0.1 => ' ',
        v if *v < 0.2 => '·',
        v if *v < 0.3 => '•',
        v if *v < 0.4 => '░',
        v if *v < 0.6 => '▒',
        v if *v < 0.8 => '▓',
        _ => '█',
    }
}

#[derive(Default)]
struct Config {
    path: String,
    resize_type: ResizeType,
    term_size: (u16,u16),
}

#[derive(Default)]
enum ResizeType {
    #[default]
    Fit,            // preserve aspect ratio
    CropToFill,     // preserve aspect ratio
    ScaleToFill,    // changes aspect ratio
}