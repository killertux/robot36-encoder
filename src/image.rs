use thiserror::Error;

const IMAGE_WIDTH: usize = 320;
const IMAGE_HEIGHT: usize = 240;
const IMAGE_SIZE: usize = IMAGE_WIDTH * IMAGE_HEIGHT;

#[derive(Error, Debug)]
pub enum ImageCreationError {
    #[error("Invalid image size {0}. Should be {1}")]
    InvalidVectorSize(usize, usize),
    #[error("Invalid width {0}. Should be {1}")]
    InvalidWidth(usize, usize),
    #[error("Invalid height {0}. Should be {1}")]
    InvalidHeight(usize, usize),
}

pub struct R(u8);
pub struct G(u8);
pub struct B(u8);
pub struct Y(u8);
pub struct U(u8);
pub struct V(u8);
/// Represents an image that will be encoded
pub struct Robot36Image {
    data: Vec<Vec<(Y, U, V)>>,
}

impl Robot36Image {
    /// Creates a Robot36Image from a RGB8 vec.
    pub fn from_rgb8_vec(vec: Vec<(R, G, B)>) -> Result<Self, ImageCreationError> {
        if vec.len() != IMAGE_SIZE {
            return Err(ImageCreationError::InvalidVectorSize(vec.len(), IMAGE_SIZE));
        }
        let mut data: Vec<Vec<(Y, U, V)>> = vec![];
        for i in 0..vec.len() {
            match data.get_mut(i / IMAGE_WIDTH) {
                Some(line) => line.push(to_yuv(&vec[i])),
                None => data.push(vec![to_yuv(&vec[i])]),
            }
        }
        Ok(Self { data })
    }

    pub fn get_height(&self) -> usize {
        IMAGE_HEIGHT
    }

    pub fn get_width(&self) -> usize {
        IMAGE_WIDTH
    }

    pub fn get_y(&self, x: usize, y: usize) -> &Y {
        &self.data[y][x].0
    }

    pub fn get_u(&self, x: usize, y: usize) -> &U {
        &self.data[y][x].1
    }

    pub fn get_v(&self, x: usize, y: usize) -> &V {
        &self.data[y][x].2
    }
}

#[cfg(feature = "image")]
impl Robot36Image {
    pub fn from_image(image: image::DynamicImage) -> Result<Robot36Image, ImageCreationError> {
        if image.width() != IMAGE_WIDTH as u32 {
            return Err(ImageCreationError::InvalidWidth(
                image.width() as usize,
                IMAGE_WIDTH,
            ));
        }
        if image.height() != IMAGE_HEIGHT as u32 {
            return Err(ImageCreationError::InvalidHeight(
                image.height() as usize,
                IMAGE_HEIGHT,
            ));
        }
        let rgb_image = image.to_rgb8().to_vec();
        let mut i = 0;
        let mut result = vec![];
        while i < rgb_image.len() {
            result.push((
                rgb_image[i].into(),
                rgb_image[i + 1].into(),
                rgb_image[i + 2].into(),
            ));
            i += 3;
        }
        Robot36Image::from_rgb8_vec(result)
    }
}

fn to_yuv(rgb: &(R, G, B)) -> (Y, U, V) {
    (
        y_rgb(rgb.0 .0, rgb.1 .0, rgb.2 .0).into(),
        u_rgb(rgb.0 .0, rgb.1 .0, rgb.2 .0).into(),
        v_rgb(rgb.0 .0, rgb.1 .0, rgb.2 .0).into(),
    )
}

fn y_rgb(r: u8, g: u8, b: u8) -> u8 {
    yuv_clamp(
        16.0 + (0.003906 * ((65.738 * r as f32) + (129.057 * g as f32) + (25.064 * b as f32))),
    )
}

fn v_rgb(r: u8, g: u8, b: u8) -> u8 {
    yuv_clamp(
        128.0 + (0.003906 * ((112.439 * r as f32) + (-94.154 * g as f32) + (-18.285 * b as f32))),
    )
}

fn u_rgb(r: u8, g: u8, b: u8) -> u8 {
    yuv_clamp(
        128.0 + (0.003906 * ((-37.945 * r as f32) + (-74.494 * g as f32) + (112.439 * b as f32))),
    )
}

fn yuv_clamp(x: f32) -> u8 {
    let tmp = if x < 0.0 { 0.0 } else { x };
    (if tmp > 255.0 { 255.0 } else { tmp }) as u8
}

macro_rules! impl_traits {
    ($i:ident) => {
        impl From<&$i> for u16 {
            fn from(value: &$i) -> u16 {
                value.0 as u16
            }
        }
        impl From<&$i> for f64 {
            fn from(value: &$i) -> f64 {
                value.0 as f64
            }
        }
        impl From<u8> for $i {
            fn from(value: u8) -> Self {
                Self(value)
            }
        }
    };
}

impl_traits!(R);
impl_traits!(G);
impl_traits!(B);
impl_traits!(Y);
impl_traits!(U);
impl_traits!(V);
