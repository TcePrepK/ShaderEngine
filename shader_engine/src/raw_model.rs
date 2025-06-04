use gl::types::{GLint, GLsizeiptr, GLuint};
use std::os::raw;
use std::ptr;

pub struct RawModel {
    id: GLuint,
    size: GLint,
}

#[allow(dead_code)]
impl RawModel {
    pub fn from_vertices(vertices: &[f32], indices: &[u32]) -> RawModel {
        let mut vbo: GLuint = 0;
        let mut ebo: GLuint = 0;
        let mut vao: GLuint = 0;

        unsafe {
            // Generate buffers
            gl::GenBuffers(1, &mut vbo);
            gl::GenBuffers(1, &mut ebo);
            gl::GenVertexArrays(1, &mut vao);

            // Bind VAO
            gl::BindVertexArray(vao);

            // Load vertex data
            gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
            gl::BufferData(
                gl::ARRAY_BUFFER,
                size_of_val(vertices) as GLsizeiptr,
                vertices.as_ptr() as *const raw::c_void,
                gl::STATIC_DRAW,
            );

            // Load index data
            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, ebo);
            gl::BufferData(
                gl::ELEMENT_ARRAY_BUFFER,
                size_of_val(indices) as GLsizeiptr,
                indices.as_ptr() as *const raw::c_void,
                gl::STATIC_DRAW,
            );

            // Setup vertex attribute pointers
            gl::EnableVertexAttribArray(0);
            gl::VertexAttribPointer(
                0,
                2,
                gl::FLOAT,
                gl::FALSE,
                2 * size_of::<f32>() as GLint,
                ptr::null(),
            );

            // Unbind buffers
            gl::BindBuffer(gl::ARRAY_BUFFER, 0);
            gl::BindVertexArray(0);
        }

        RawModel {
            id: vbo,
            size: indices.len() as GLint,
        }
    }

    pub fn render(&self) {
        unsafe {
            gl::BindVertexArray(self.id);
            gl::DrawElements(gl::TRIANGLES, self.size, gl::UNSIGNED_INT, ptr::null());
            gl::BindVertexArray(0);
        }
    }
}
