## Robot36-encoder

This is a simple encoder of images into audio samples by using the Robot 36 encoding. It is based on the C implementation of [Ahmet](https://github.com/xdsopl/robot36). You can use his decoder to decode this image.

It works as a library so you can use in your rust project. You encode a Robot36Image into an iterator of i16 representing each sample of the audio. You can then use this sample to write to an audio file or directly into the audio output. There is an optinal feature `image`, that can be enabled to add a conversion function from [DynamicImage](https://docs.rs/image/latest/image/enum.DynamicImage.html) into the Robot36Image.

Bear in mind that breaking changes may and probably will happen in next versions!

## Example using Hound

```rust
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
```
