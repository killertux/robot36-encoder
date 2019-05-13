## Robot36-encoder

This is a simple encoder of images into audio samples by using the Robot 36 encoding. It is based on the C implementation of [Ahmet](https://github.com/xdsopl/robot36)

It works as a library so you can use in your rust project. You encode a [DynamicImage](https://docs.rs/image) into a vector of vector of i16 representing each sample of the audio. You can then use this sample to write to an audio file or directly into the audio output.

Bear in mind that this is still on the 0.1 version so breaking changes may and probably will happen!

## Example using Hound

'''rust
extern crate robot36-encoder;

use std::env;
use encoder::Encoder;

/** Simple example.
 * encode-image /path/to/image-input.png /path/to/audio-output.wav
*/
fn main() {
    let args: Vec<String> = env::args().collect();
    let image = image::open(args[1].to_string()).unwrap();
    let mut encoder = Encoder::new(image, 48000);
    let samples = encoder.encode().unwrap();
    let spec = hound::WavSpec {
        channels: 1,
        sample_rate: 48000,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };
    let mut writer = hound::WavWriter::create(args[2].to_string(), spec).unwrap();
    for sample in samples.iter() {
        writer.write_sample(*sample).unwrap();
    }
}
'''