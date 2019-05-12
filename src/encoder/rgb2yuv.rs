pub fn y_rgb(r: u8, g: u8, b: u8) -> u8 {
    yuv_clamp(16.0 + (0.003906 * ((65.738 * r as f32) + (129.057 * g as f32) + (25.064 * b as f32))))
}

pub fn v_rgb(r: u8, g: u8, b: u8) -> u8 {
    yuv_clamp(128.0 + (0.003906 * ((112.439 * r as f32) + (-94.154 * g as f32) + (-18.285 * b as f32))))
}

pub fn u_rgb(r: u8, g: u8, b: u8) -> u8 {
    yuv_clamp(128.0 + (0.003906 * ((-37.945 * r as f32) + (-74.494 * g as f32) + (112.439 * b as f32))))
}

fn yuv_clamp(x: f32) -> u8 {
    let tmp = if x < 0.0 {0.0} else {x};
    (if tmp > 255.0 {255.0} else {tmp}) as u8
}
