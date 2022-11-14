use super::VertexArray;

pub struct Buffer<T> {
    id: u32,
    data: Vec<T>
}

impl<T> Buffer<T> {
    pub fn new() -> Buffer<T> {
        let mut buffer = Buffer {
            data: Vec::<T>::new(), id: 0
        };

        unsafe {
            gl::GenBuffers(1, &mut buffer.id);
        }

        buffer
    }

    // VAO is needed as obj is part of it
    pub fn send_data_static(&mut self, vao: &VertexArray, data: Vec<T>, target: gl::types::GLenum) {
        vao.bind_array();
        self.data = data;

        unsafe {
            gl::BindBuffer(target, self.id);
            gl::BufferData(
                target,
                (self.data.len() * std::mem::size_of::<T>()) as isize,
                self.data.as_ptr() as *const gl::types::GLvoid,
                gl::STATIC_DRAW
            );
        }

        VertexArray::unbind_all();
    }

    pub fn len(&self) -> usize {
        self.data.len()
    }

    // TODO: add ways to change these during runtime,
    // TODO: like for example changing transforms

    // TODO: multiple buffer gen function
}

impl<T> Drop for Buffer<T> {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteBuffers(1, &self.id);
        }
    }
}