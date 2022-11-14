pub struct VertexArray {
    id: u32,
    index: u32
}

impl VertexArray {
    pub fn new() -> VertexArray {
        let mut vert_array = VertexArray {
            id: 0, index: 0
        };

        unsafe {
            gl::GenVertexArrays(1, &mut vert_array.id);
        }

        vert_array
    }

    // TODO: Gen multiple function?

    pub fn bind_array(&self) {
        unsafe {
            gl::BindVertexArray(self.id);
        }
    }

    pub fn add_attrib(&mut self, size: i32, stride: i32, pointer: *const gl::types::GLvoid) {
        unsafe {
            // Binding this each call is technically slower, but it's safer and only affects loading
            gl::BindVertexArray(self.id);

            gl::EnableVertexAttribArray(self.index);
            gl::VertexAttribPointer(
                self.index,
                size,
                gl::FLOAT,
                gl::FALSE,
                stride,
                pointer
            );

            gl::BindVertexArray(0);
        }

        self.index += 1;
    }

    // For adding things like mat4 (types that are larger than 4*f32s but are multiples of it)
    pub fn add_attrib_divisor(&mut self, rows: i32) {
        // Row size is constant in OpenGL
        let size_vec4 = 16;

        unsafe {
            gl::BindVertexArray(self.id);

            for i in 0..rows {
                gl::EnableVertexAttribArray(self.index);
                gl::VertexAttribPointer(
                    self.index,
                    4,
                    gl::FLOAT,
                    gl::FALSE,
                    size_vec4 * rows,
                    (i as i32 * size_vec4) as *const gl::types::GLvoid
                );
                gl::VertexAttribDivisor(self.index, 1);

                self.index += 1;
            }

            gl::BindVertexArray(0);
        }
    }

    // Get count and instance_count from in-built buffer objects
    pub fn draw_elements(&self, count: i32, instance_count: i32) {
        unsafe {
            gl::BindVertexArray(self.id);
            gl::DrawElementsInstanced(
                gl::TRIANGLES,
                count,
                gl::UNSIGNED_INT,
                std::ptr::null(),
                instance_count
            );
            gl::BindVertexArray(0);
        }
    }

    pub fn unbind_all() {
        unsafe { gl::BindVertexArray(0) }
    }
}

impl Drop for VertexArray {
    fn drop(&mut self) {
        unsafe {
            gl::BindVertexArray(0);
            gl::DeleteVertexArrays(1, &self.id);
        }
    }
}