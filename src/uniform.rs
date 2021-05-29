use cfg_if::cfg_if;

/// Types that can be used as uniform argument type.
pub trait UniformType: Copy + 'static {
    /// Applies the uniform value to the specified location.
    fn apply(
        self,
        context: &web_sys::WebGlRenderingContext,
        location: &web_sys::WebGlUniformLocation,
    );
}

macro_rules! uniform_type {
    ($ty:ty, $method:ident, |$x:ident| ($($value:expr),*)) => {
        impl UniformType for $ty {
            fn apply(self, context: &web_sys::WebGlRenderingContext, location: &web_sys::WebGlUniformLocation) {
                let $x = self;
                context.$method(Some(location), $($value),*);
            }
        }
    }
}

uniform_type!(i32, uniform1i, |x| (x));
uniform_type!(f32, uniform1f, |x| (x));

uniform_type!((i32, i32), uniform2i, |x| (x.0, x.1));
uniform_type!((f32, f32), uniform2f, |x| (x.0, x.1));

uniform_type!((i32, i32, i32), uniform3i, |x| (x.0, x.1, x.2));
uniform_type!((f32, f32, f32), uniform3f, |x| (x.0, x.1, x.2));

uniform_type!((i32, i32, i32, i32), uniform4i, |x| (x.0, x.1, x.2, x.3));
uniform_type!((f32, f32, f32, f32), uniform4f, |x| (x.0, x.1, x.2, x.3));

cfg_if! {
    if #[cfg(feature = "nalgebra")] {
        uniform_type!(nalgebra::Vector2<i32>, uniform2i, |x| (x[0], x[1]));
        uniform_type!(nalgebra::Vector2<f32>, uniform2f, |x| (x[0], x[1]));

        uniform_type!(nalgebra::Vector3<i32>, uniform3i, |x| (x[0], x[1], x[2]));
        uniform_type!(nalgebra::Vector3<f32>, uniform3f, |x| (x[0], x[1], x[2]));

        uniform_type!(nalgebra::Vector4<i32>, uniform4i, |x| (x[0], x[1], x[2], x[3]));
        uniform_type!(nalgebra::Vector4<f32>, uniform4f, |x| (x[0], x[1], x[2], x[3]));

        uniform_type!(nalgebra::Matrix2<f32>, uniform_matrix2fv_with_f32_array, |x| (false, x.as_slice()));
        uniform_type!(nalgebra::Matrix3<f32>, uniform_matrix3fv_with_f32_array, |x| (false, x.as_slice()));
        uniform_type!(nalgebra::Matrix4<f32>, uniform_matrix4fv_with_f32_array, |x| (false, x.as_slice()));
    }
}
