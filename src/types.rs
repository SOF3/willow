use cfg_if::cfg_if;
use web_sys::{WebGlRenderingContext, WebGlUniformLocation};

/// Types that can be used as a uniform argument type.
pub trait UniformType: Sized + Copy + 'static {
    /// Applies the uniform value to the specified location.
    fn apply_uniform(self, context: &WebGlRenderingContext, location: &WebGlUniformLocation);
}

macro_rules! impl_uniform {
    ($ty:ty; $method:ident, |$x:ident| ($($value:expr),*)) => {
        impl UniformType for $ty {
            fn apply_uniform(self, context: &WebGlRenderingContext, location: &WebGlUniformLocation) {
                let $x = self;
                context.$method(Some(location), $($value),*);
            }
        }
    }
}

/// Types that can be used as an attribute argument type.
pub trait AttributeType: Sized + Copy + 'static {
    /// Number of components in the type.
    fn num_comps() -> usize;

    /// Corresponding GLenum specifying the data type of each component in the array,
    /// e.g. `WebGlRenderingContext::UNSIGNED_SHORT`.
    fn gl_type() -> u32;
}

macro_rules! impl_attribute {
    ($ty:ty; $glty:ident) => {
        impl AttributeType for $ty {
            fn num_comps() -> usize {
                1
            }

            fn gl_type() -> u32 {
                WebGlRenderingContext::$glty
            }
        }

        impl AttributeType for ($ty, $ty) {
            fn num_comps() -> usize {
                2
            }

            fn gl_type() -> u32 {
                WebGlRenderingContext::$glty
            }
        }

        impl AttributeType for ($ty, $ty, $ty) {
            fn num_comps() -> usize {
                3
            }

            fn gl_type() -> u32 {
                WebGlRenderingContext::$glty
            }
        }

        impl AttributeType for ($ty, $ty, $ty, $ty) {
            fn num_comps() -> usize {
                4
            }

            fn gl_type() -> u32 {
                WebGlRenderingContext::$glty
            }
        }

        #[cfg(feature = "nalgebra")]
        impl AttributeType for nalgebra::Vector2<$ty> {
            fn num_comps() -> usize {
                2
            }

            fn gl_type() -> u32 {
                WebGlRenderingContext::$glty
            }
        }

        #[cfg(feature = "nalgebra")]
        impl AttributeType for nalgebra::Vector3<$ty> {
            fn num_comps() -> usize {
                3
            }

            fn gl_type() -> u32 {
                WebGlRenderingContext::$glty
            }
        }

        #[cfg(feature = "nalgebra")]
        impl AttributeType for nalgebra::Vector4<$ty> {
            fn num_comps() -> usize {
                4
            }

            fn gl_type() -> u32 {
                WebGlRenderingContext::$glty
            }
        }
    };
}

impl_attribute!(i8; BYTE);
impl_attribute!(i16; SHORT);
impl_attribute!(u8; UNSIGNED_BYTE);
impl_attribute!(u16; UNSIGNED_SHORT);
impl_attribute!(f32; FLOAT);

impl_uniform!(i32; uniform1i, |x| (x));
impl_uniform!(f32; uniform1f, |x| (x));

impl_uniform!((i32, i32); uniform2i, |x| (x.0, x.1));
impl_uniform!((f32, f32); uniform2f, |x| (x.0, x.1));

impl_uniform!((i32, i32, i32); uniform3i, |x| (x.0, x.1, x.2));
impl_uniform!((f32, f32, f32); uniform3f, |x| (x.0, x.1, x.2));

impl_uniform!((i32, i32, i32, i32); uniform4i, |x| (x.0, x.1, x.2, x.3));
impl_uniform!((f32, f32, f32, f32); uniform4f, |x| (x.0, x.1, x.2, x.3));

cfg_if! {
    if #[cfg(feature = "nalgebra")] {
        impl_uniform!(nalgebra::Vector2<i32>; uniform2i, |x| (x[0], x[1]));
        impl_uniform!(nalgebra::Vector2<f32>; uniform2f, |x| (x[0], x[1]));

        impl_uniform!(nalgebra::Vector3<i32>; uniform3i, |x| (x[0], x[1], x[2]));
        impl_uniform!(nalgebra::Vector3<f32>; uniform3f, |x| (x[0], x[1], x[2]));

        impl_uniform!(nalgebra::Vector4<i32>; uniform4i, |x| (x[0], x[1], x[2], x[3]));
        impl_uniform!(nalgebra::Vector4<f32>; uniform4f, |x| (x[0], x[1], x[2], x[3]));

        impl_uniform!(nalgebra::Matrix2<f32>; uniform_matrix2fv_with_f32_array, |x| (false, x.as_slice()));
        impl_uniform!(nalgebra::Matrix3<f32>; uniform_matrix3fv_with_f32_array, |x| (false, x.as_slice()));
        impl_uniform!(nalgebra::Matrix4<f32>; uniform_matrix4fv_with_f32_array, |x| (false, x.as_slice()));
    }
}
