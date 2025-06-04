use gl::types::{GLenum, GLint, GLsizei, GLuint};

macro_rules! image_texture_type {
    (
        $name:ident,
        $gl_target:expr,
        $gl_texture_fn:ident,
        [$($dim:ident),+]
    ) => {
        #[allow(dead_code)]
        pub struct $name {
            pub texture_id: GLuint,
            $(pub $dim: i32,)+
            access: GLenum,
            internal_format: GLenum,
            format: GLenum,
        }

        #[allow(dead_code)]
        impl $name {
            #[allow(clippy::too_many_arguments)]
            pub fn new(
                $($dim: i32,)+
                access: GLenum,
                internal_format: GLenum,
                image_format: GLenum,
            ) -> Self {
                let mut texture_id = 0;
                unsafe {
                    gl::CreateTextures($gl_target, 1, &mut texture_id);

                    gl::TextureParameteri(texture_id, gl::TEXTURE_MIN_FILTER, gl::NEAREST as GLint);
                    gl::TextureParameteri(texture_id, gl::TEXTURE_MAG_FILTER, gl::NEAREST as GLint);
                    if $gl_target == gl::TEXTURE_2D || $gl_target == gl::TEXTURE_3D {
                        gl::TextureParameteri(texture_id, gl::TEXTURE_WRAP_S, gl::CLAMP_TO_EDGE as GLint);
                    }
                    if $gl_target == gl::TEXTURE_3D {
                        gl::TextureParameteri(texture_id, gl::TEXTURE_WRAP_T, gl::CLAMP_TO_EDGE as GLint);
                    }

                    gl::$gl_texture_fn(
                        texture_id,
                        1,
                        internal_format,
                        $($dim as GLsizei,)+
                    );
                    gl::BindImageTexture(0, texture_id, 0, gl::FALSE, 0, access, image_format);
                }

                Self {
                    texture_id,
                    $($dim,)+
                    access,
                    internal_format,
                    format: image_format,
                }
            }

            pub fn clone_with_size(&self, $($dim: i32,)+) -> $name {
                $name::new($($dim,)+ self.access, self.internal_format, self.format)
            }

            pub fn bind_as_image(&self, unit: GLuint) {
                unsafe {
                    gl::BindImageTexture(
                        unit,
                        self.texture_id,
                        0,
                        gl::FALSE,
                        0,
                        self.access,
                        self.format,
                    );
                }
            }

            pub fn bind_as_sampler(&self, unit: GLuint) {
                unsafe {
                    gl::ActiveTexture(gl::TEXTURE0 + unit as GLenum);
                    gl::BindTexture($gl_target, self.texture_id);
                }
            }
        }
    };
}

image_texture_type!(Image1D, gl::TEXTURE_1D, TextureStorage1D, [width]);
image_texture_type!(Image2D, gl::TEXTURE_2D, TextureStorage2D, [width, height]);
image_texture_type!(Image3D, gl::TEXTURE_3D, TextureStorage3D, [
    width, height, depth
]);
