use crate::image::Robot36Image;
use core::ops::Mul;
use num_complex::Complex;

/// Encoder that will encode the image into an sampled audio.
pub struct Encoder {
    image: Robot36Image,
    rate: u64,
    y_ticks: u64,
    uv_ticks: u64,
    sync_porch_ticks: u64,
    porch_ticks: u64,
    horizontal_sync_ticks: u64,
    seperator_ticks: u64,
}

const SYNC_PORCH_SECS: f64 = 0.003;
const PORCH_SECS: f64 = 0.0015;
const Y_SECS: f64 = 0.088;
const UV_SECS: f64 = 0.044;
const HORIZONTAL_SYNC_SECS: f64 = 0.009;
const SEPARATOR_SECS: f64 = 0.0045;

impl Encoder {
    /// Creates a new encoder.
    /// The rate is the sampling rate of the output audio
    pub fn new(image: Robot36Image, rate: u64) -> Self {
        Encoder {
            image,
            rate: rate,
            y_ticks: (rate as f64 * Y_SECS) as u64,
            uv_ticks: (rate as f64 * UV_SECS) as u64,
            sync_porch_ticks: (rate as f64 * SYNC_PORCH_SECS) as u64,
            porch_ticks: (rate as f64 * PORCH_SECS) as u64,
            horizontal_sync_ticks: (rate as f64 * HORIZONTAL_SYNC_SECS) as u64,
            seperator_ticks: (rate as f64 * SEPARATOR_SECS) as u64,
        }
    }

    /// Encodes the image into an iterator of i16. Where each number is
    /// a audio sample.
    pub fn encode<'a>(&'a self) -> impl Iterator<Item = i16> + 'a {
        Signal::new(
            ComplexOscilator::new(self.rate),
            self.vis_header().chain(self.codificated_image()),
        )
    }

    fn vis_header<'a>(&'a self) -> impl Iterator<Item = f64> + 'a {
        let header_frequencies: Vec<(f64, f64)> = vec![
            (self.rate as f64 * 0.3, 0.0),
            (1900.0, 0.3),
            (1200.0, 0.01),
            (1900.0, 0.3),
            (1200.0, 0.03),
            (1300.0, 0.03),
            (1300.0, 0.03),
            (1300.0, 0.03),
            (1100.0, 0.03),
            (1300.0, 0.03),
            (1300.0, 0.03),
            (1300.0, 0.03),
            (1100.0, 0.03),
            (1200.0, 0.03),
        ];
        header_frequencies.into_iter().flat_map(|frequency| {
            get_frequency_iter((self.rate as f64 * frequency.1) as u64, frequency.0)
        })
    }

    fn codificated_image<'a>(&'a self) -> impl Iterator<Item = f64> + 'a {
        (0..self.image.get_height()).step_by(2).flat_map(|y| {
            get_frequency_iter(self.horizontal_sync_ticks, 1200.0)
                .chain(get_frequency_iter(self.sync_porch_ticks, 1500.0))
                .chain(self.add_y_scan(y))
                .chain(get_frequency_iter(self.seperator_ticks, 1500.0))
                .chain(get_frequency_iter(self.porch_ticks, 1900.0))
                .chain(self.add_v_scan(y))
                .chain(get_frequency_iter(self.horizontal_sync_ticks, 1200.0))
                .chain(get_frequency_iter(self.sync_porch_ticks, 1500.0))
                .chain(self.add_y_scan(y + 1))
                .chain(get_frequency_iter(self.seperator_ticks, 2300.0))
                .chain(get_frequency_iter(self.porch_ticks, 1900.0))
                .chain(self.add_u_scan(y + 1))
        })
    }

    fn add_y_scan<'a>(&'a self, y: usize) -> impl Iterator<Item = f64> + 'a {
        (0..self.y_ticks).map(move |tick| {
            let x = self.get_x_position(tick, self.y_ticks);
            1500.0 + 800.0 * f64::from(self.image.get_y(x, y)) / 255.0
        })
    }

    fn add_v_scan<'a>(&'a self, y: usize) -> impl Iterator<Item = f64> + 'a {
        (0..self.uv_ticks).map(move |tick| {
            let x0 = self.get_x_position(tick, self.uv_ticks);
            let x1 = (x0 + 1).max(self.image.get_width() - 1);
            let yuv_v = ((u16::from(self.image.get_v(x0, y)) + u16::from(self.image.get_v(x1, y)))
                / 2) as f64;
            1500.0 + 800.0 * (yuv_v / 255.0)
        })
    }

    fn add_u_scan<'a>(&'a self, y: usize) -> impl Iterator<Item = f64> + 'a {
        (0..self.uv_ticks).map(move |tick| {
            let x0 = self.get_x_position(tick, self.uv_ticks);
            let x1 = (x0 + 1).max(self.image.get_width() - 1);
            let yuv_u = ((u16::from(self.image.get_u(x0, y - 1))
                + u16::from(self.image.get_u(x1, y - 1)))
                / 2) as f64;
            1500.0 + 800.0 * (yuv_u / 255.0)
        })
    }

    fn get_x_position(&self, tick: u64, ticks: u64) -> usize {
        ((self.image.get_width() - 1) as f64 * tick as f64 / ticks as f64) as usize
    }
}

fn get_frequency_iter(number_of_ticks: u64, frequency: f64) -> impl Iterator<Item = f64> {
    (0..number_of_ticks).map(move |_| frequency)
}

struct ComplexOscilator {
    complex_number: Complex<f64>,
    hz_2_rad: f64,
}

impl ComplexOscilator {
    pub fn new(rate: u64) -> Self {
        Self {
            complex_number: Complex::new(0.5, 0.5),
            hz_2_rad: (2.0 * std::f64::consts::PI as f64) / rate as f64,
        }
    }

    pub fn add_freq(&mut self, freq: f64) {
        let exponetial: Complex<f64> = Complex::new(0.0, freq * self.hz_2_rad);
        self.complex_number = self.complex_number.mul(exponetial.exp());
    }
}

struct Signal<T>
where
    T: Iterator<Item = f64>,
{
    complex_oscilator: ComplexOscilator,
    frequencies: T,
}

impl<T> Signal<T>
where
    T: Iterator<Item = f64>,
{
    pub fn new(complex_oscilator: ComplexOscilator, frequencies: T) -> Self {
        Self {
            complex_oscilator,
            frequencies: frequencies,
        }
    }
}

impl<T> Iterator for Signal<T>
where
    T: Iterator<Item = f64>,
{
    type Item = i16;
    fn next(&mut self) -> Option<Self::Item> {
        match self.frequencies.next() {
            None => None,
            Some(freq) => {
                self.complex_oscilator.add_freq(freq);
                Some((self.complex_oscilator.complex_number.re * std::i16::MAX as f64) as i16)
            }
        }
    }
}
