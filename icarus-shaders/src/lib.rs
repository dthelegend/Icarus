#![no_std]
#![allow(unexpected_cfgs)]

use spirv_std::glam::{Vec3, Vec4, vec4};
use spirv_std::spirv;

#[spirv(fragment)]
pub fn main_fs(output: &mut Vec4) {
    *output = vec4(1.0, 0.0, 0.0, 1.0);
}

#[spirv(vertex)]
pub fn main_vs(
    in_position: Vec3,
    #[spirv(position)] gl_position: &mut Vec4,
) {
    *gl_position = in_position.extend(1.0);
}