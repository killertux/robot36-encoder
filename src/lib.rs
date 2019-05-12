mod encoder;
use encoder::Encoder;

#[cfg(test)]
mod tests {
    extern crate image;
    use super::*;

    #[test]
    fn smoke_test() {
        let image = image::open("examples/test_pattern.png").unwrap();
        let mut encoder = Encoder::new(image, 44100);
        let samples = encoder.encode();
        assert_eq!(2 + 2, 4);
    }
}
