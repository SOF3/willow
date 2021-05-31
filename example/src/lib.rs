use nalgebra::{Matrix4, Vector3};
use willow::{Attribute, Program, ProgramData, Uniform};

#[derive(Program)]
#[willow(path = "foo")]
pub struct Foo {
    data: ProgramData,
    u_alpha: Uniform<f32>,
    u_transform: Uniform<Matrix4<f32>>,
    a_color: Attribute<Vector3<f32>>,
    a_offset: Attribute<Vector3<f32>>,
}
