extern crate image;
extern crate num;

mod rgb2yuv;

use num::complex::Complex;
use core::ops::Mul;
use image::{DynamicImage, FilterType};

pub struct Encoder {
    image_as_bytes: Vec<u8>,
    buffer: Vec<i16>,
    complex_signal: Complex<f32>,
    hz_2_rad: f32,
    rate: u32,
    y_ticks: u32,
    uv_ticks: u32,
    sync_porch_ticks: u32,
    porch_ticks: u32,
    horizontal_sync_ticks: u32,
    seperator_ticks: u32,
}

const SYNC_PORCH_SECS: f32 = 0.003;
const PORCH_SECS: f32 = 0.0015;
const Y_SECS: f32 = 0.088;
const UV_SECS: f32 = 0.044;
const HORIZONTAL_SYNC_SECS: f32 = 0.009;
const SEPARATOR_SECS: f32 = 0.0045;
const IMAGE_WIDTH: u32 = 320;
const IMAGE_HEIGHT: u32 = 240;

impl Encoder {
    pub fn new(image: DynamicImage, rate: u32) -> Self {
        let rgb_image = image.resize(IMAGE_WIDTH, IMAGE_HEIGHT, FilterType::Lanczos3).to_rgb();
        Encoder {
            image_as_bytes: rgb_image.into_vec(),
            buffer: Vec::new(),
            complex_signal: Complex::new(0.5, 0.5),
            hz_2_rad: (2.0 * std::f64::consts::PI as f32) / rate as f32,
            rate: rate,
            y_ticks: (rate as f32 * Y_SECS) as u32,
            uv_ticks: (rate as f32 * UV_SECS) as u32,
            sync_porch_ticks: (rate as f32 * SYNC_PORCH_SECS) as u32,
            porch_ticks: (rate as f32 * PORCH_SECS) as u32,
            horizontal_sync_ticks: (rate as f32 * HORIZONTAL_SYNC_SECS) as u32,
            seperator_ticks: (rate as f32 * SEPARATOR_SECS) as u32,
        }
    }

    pub fn encode(self: &mut Self) -> Result<Vec<i16>, String> {
        self.add_vis_header();
        self.add_codificated_image();
        Ok(self.buffer.clone())
    }

    fn add_codificated_image(self: &mut Self) {
        let mut y = 0;
        while y < IMAGE_HEIGHT {
            self.add_signal(self.horizontal_sync_ticks, 1200.0);
            self.add_signal(self.sync_porch_ticks, 1500.0);
            self.add_y_scan(&y);
            self.add_signal(self.seperator_ticks, 1500.0);
            self.add_signal(self.porch_ticks, 1900.0);
            self.add_v_scan(&y);
            y += 1;
            self.add_signal(self.horizontal_sync_ticks, 1200.0);
            self.add_signal(self.sync_porch_ticks, 1500.0);
            self.add_y_scan(&y);
            self.add_signal(self.seperator_ticks, 2300.0);
            self.add_signal(self.porch_ticks, 1900.0);
            self.add_u_scan(&y);
            y += 1;
        }
    }

    fn add_signal(self: &mut Self, number_of_ticks: u32, frequency: f32) {
        for _i in 0..number_of_ticks {
            self.add_freq(&frequency);
        }
    }

    fn add_sample(self: &mut Self, value: &f32) {
        self.buffer.push((value * std::i16::MAX as f32) as i16);
    }

    fn add_freq(self: &mut Self, value: &f32) {
        let real: f32 = self.complex_signal.re;
        self.add_sample(&real);
        let exponetial: Complex<f32> = Complex::new(0.0, value * self.hz_2_rad);
        self.complex_signal = self.complex_signal.mul(exponetial.exp());
    }

    fn add_y_scan(self: &mut Self, y: &u32) {
        for ticks in 0..self.y_ticks {
            let x: f32 = self.fclampf(320.0 * ticks as f32 / self.y_ticks as f32, 0.0, 319.0);
            let off0: u32 = ((3 * y * IMAGE_WIDTH) as f32 + 3.0 * x) as u32;
            let r0 = self.image_as_bytes[(off0 + 0) as usize];
            let g0 = self.image_as_bytes[(off0 + 1) as usize];
            let b0 = self.image_as_bytes[(off0 + 2) as usize];
            self.add_freq(&(1500.0 + 800.0 * rgb2yuv::y_rgb(r0, g0, b0) as f32 / 255.0));
        }
    }

    fn add_v_scan(self: &mut Self, y: &u32) {
        for ticks in 0..self.uv_ticks {
            let x0: u32 = self.fclampf(160.0 * ticks as f32 / self.uv_ticks as f32, 0.0, 159.0) as u32;
            let x1: u32 = self.fclampf((x0 + 1) as f32, 0.0, 159.0) as u32;
            let evn0: u32 = 3 * y * IMAGE_WIDTH + 6 * x0;
            let evn1: u32 = 3 * y * IMAGE_WIDTH + 6 * x1;
            let r0 = self.image_as_bytes[(evn0 + 0) as usize];
            let g0 = self.image_as_bytes[(evn0 + 1) as usize];
            let b0 = self.image_as_bytes[(evn0 + 2) as usize];
            let r1 = self.image_as_bytes[(evn1 + 0) as usize];
            let g1 = self.image_as_bytes[(evn1 + 1) as usize];
            let b1 = self.image_as_bytes[(evn1 + 2) as usize];
            let yuv_v = ((rgb2yuv::v_rgb(r0, g0, b0) as u16 + rgb2yuv::v_rgb(r1, g1, b1) as u16) / 2) as f32;
            self.add_freq(&(1500.0 + 800.0 * yuv_v as f32 / 255.0));
        }
    }

    fn add_u_scan(self: &mut Self, y: &u32) {
        for ticks in 0..self.uv_ticks {
            let x0: u32 = self.fclampf(160.0 * ticks as f32 / self.uv_ticks as f32, 0.0, 159.0) as u32;
            let x1: u32 = self.fclampf((x0 + 1) as f32, 0.0, 159.0) as u32;
            let evn0: u32 = 3 * (y - 1) * IMAGE_WIDTH + 6 * x0;
            let evn1: u32 = 3 * (y - 1) * IMAGE_WIDTH + 6 * x1;
            let r0 = self.image_as_bytes[(evn0 + 0) as usize];
            let g0 = self.image_as_bytes[(evn0 + 1) as usize];
            let b0 = self.image_as_bytes[(evn0 + 2) as usize];
            let r1 = self.image_as_bytes[(evn1 + 0) as usize];
            let g1 = self.image_as_bytes[(evn1 + 1) as usize];
            let b1 = self.image_as_bytes[(evn1 + 2) as usize];
            let yuv_u = ((rgb2yuv::u_rgb(r0, g0, b0) as u16 + rgb2yuv::u_rgb(r1, g1, b1) as u16) / 2) as f32;
            self.add_freq(&(1500.0 + 800.0 * yuv_u / 255.0));
        }
    }

    fn add_vis_header(self: &mut Self) {
        self.add_signal((self.rate as f32 * 0.3) as u32, 0.0);
	    let header_frequencies: Vec<(f32, f32)> = vec!(
            (1900.0, 0.3),
            (1200.0, 0.01),
            (1900.0, 0.3),
            (1200.0, 0.03),
            (1100.0, 0.03),
            (1300.0, 0.03),
            (1100.0, 0.03),
            (1100.0, 0.03),
            (1300.0, 0.03),
            (1300.0, 0.03),
            (1300.0, 0.03),
            (1100.0, 0.03),
            (1200.0, 0.03)
        );
        for frequency in header_frequencies {
            self.add_signal((self.rate as f32 * frequency.1) as u32, frequency.0)
        }
    }

    fn fclampf(self: &Self, x: f32, min: f32, max: f32) -> f32 {
       let tmp = if x < min {min} else {x};
        if tmp > max {max} else {x}
    }
}
