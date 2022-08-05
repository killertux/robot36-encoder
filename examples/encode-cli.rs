use robot36_encoder::{Encoder, Robot36Image};
use std::env;

/** Simple exmaple.
 * encode-image /path/to/image-input.png /path/to/audio-output.wav
*/
fn main() {
    let args: Vec<String> = env::args().collect();
    let image = image::open(args[1].to_string()).unwrap();
    let encoder = Encoder::new(
        Robot36Image::from_image(image.resize(320, 240, image::imageops::FilterType::Lanczos3))
            .unwrap(),
        48000,
    );
    let samples = encoder.encode();
    let spec = hound::WavSpec {
        channels: 1,
        sample_rate: 48000,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };
    let mut writer = hound::WavWriter::create(args[2].to_string(), spec).unwrap();
    for sample in samples {
        writer.write_sample(sample).unwrap();
    }
}
