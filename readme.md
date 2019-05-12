## Robot36-encoder

This is a simple encoder of images into audio samples by using the Robot 36 encoding. It is based on the implementationof [Ahmet][https://github.com/xdsopl/robot36]

It works as a library so you can use in your rust project. You encode a [DynamicImage][https://docs.rs/image] into a vector of vector of i16 representing each sample of the audio. You can then use this sample to write to an audio file or directly into the audio output.