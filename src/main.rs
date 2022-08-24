extern crate gl;
extern crate glfw;
extern crate image;

mod shader_program;

use self::glfw::{Context, Key, Action};
use std::sync::mpsc::Receiver;
use shader_program::ShaderProgram;
use std::path::Path;
use std::ffi::CString;

fn main() {
    let mut glfw: glfw::Glfw = glfw::init(glfw::FAIL_ON_ERRORS).unwrap();
    glfw.window_hint(glfw::WindowHint::ContextVersion(3, 3));
    glfw.window_hint(glfw::WindowHint::OpenGlProfile(glfw::OpenGlProfileHint::Core));
    #[cfg(target_os = "macos")] glfw.window_hint(glfw::WindowHint::OpenGlForwardCompat(true));

    let (mut window, events) = glfw.create_window(
        800,
        600,
        "LearnOpenGL",
        glfw::WindowMode::Windowed
    ).expect("Failed to create GLFW window");

    window.make_current();
    window.set_key_polling(true);
    window.set_framebuffer_size_polling(true);

    gl::load_with(|symbol| window.get_proc_address(symbol) as *const _);
    
    // Triangle
    let vertices: Vec<f32> = vec![
        // positions      // colors        // texture coords
        0.5,  0.5, 0.0,   1.0, 0.0, 0.0,   1.0, 1.0,   // top right
        0.5, -0.5, 0.0,   0.0, 1.0, 0.0,   1.0, 0.0,   // bottom right
       -0.5, -0.5, 0.0,   0.0, 0.0, 1.0,   0.0, 0.0,   // bottom left
       -0.5,  0.5, 0.0,   1.0, 1.0, 0.0,   0.0, 1.0    // top left 
    ];

    let indices = [
        0, 1, 3,  // first Triangle
        1, 2, 3   // second Triangle
    ];

    let (shader_program, vao, vbo, ebo, texture1, texture2) = unsafe {
        let shader_program = ShaderProgram::new(
            "assets/shaders/shader.vert",
            "assets/shaders/shader.frag"
        );

        let (mut vao, mut vbo, mut ebo) = (0, 0, 0);
        gl::GenVertexArrays(1, &mut vao);
        gl::GenBuffers(1, &mut vbo);
        gl::GenBuffers(1, &mut ebo);

        gl::BindVertexArray(vao);

        gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
        gl::BufferData(
            gl::ARRAY_BUFFER,
            (vertices.len() * std::mem::size_of::<f32>()) as gl::types::GLsizeiptr,
            vertices.as_ptr() as *const gl::types::GLvoid,
            gl::STATIC_DRAW
        );
        
        gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, ebo);
        gl::BufferData(
            gl::ELEMENT_ARRAY_BUFFER,
            (indices.len() * std::mem::size_of::<f32>()) as gl::types::GLsizeiptr,
            indices.as_ptr() as *const gl::types::GLvoid,
            gl::STATIC_DRAW
        );

        let stride = (8 * std::mem::size_of::<f32>()) as gl::types::GLint;

        // Vertex coords
        gl::VertexAttribPointer(
            0,
            3,
            gl::FLOAT,
            gl::FALSE,
            stride,
            std::ptr::null()
        );
        gl::EnableVertexAttribArray(0);

        // Color
        gl::VertexAttribPointer(
            1,
            3,
            gl::FLOAT,
            gl::FALSE,
            stride,
            (3 * std::mem::size_of::<f32>()) as *const gl::types::GLvoid
        );
        gl::EnableVertexAttribArray(1);

        // Texture coords
        gl::VertexAttribPointer(
            2,
            2,
            gl::FLOAT,
            gl::FALSE,
            stride,
            (6 * std::mem::size_of::<f32>()) as *const gl::types::GLvoid
        );
        gl::EnableVertexAttribArray(2);

        gl::BindBuffer(gl::ARRAY_BUFFER, 0);
        gl::BindVertexArray(0);

        // Uncomment to enable wireframe mode
        // gl::PolygonMode(gl::FRONT_AND_BACK, gl::LINE);

        // Load textures
        let mut texture1 = 0;
        gl::GenTextures(1, &mut texture1);
        gl::BindTexture(gl::TEXTURE_2D, texture1);

        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::REPEAT as i32);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::REPEAT as i32);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR_MIPMAP_LINEAR as i32);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32);

        let img = image::open(&Path::new("assets/textures/container.jpg")).expect("Failed to load texture");
        let data = img.as_bytes();

        gl::TexImage2D(
            gl::TEXTURE_2D,
            0,
            gl::RGB as i32,
            img.width() as i32,
            img.height() as i32,
            0,
            gl::RGB,
            gl::UNSIGNED_BYTE,
            data.as_ptr() as *const gl::types::GLvoid
        );
        gl::GenerateMipmap(gl::TEXTURE_2D);

        let mut texture2 = 0;
        gl::GenTextures(1, &mut texture2);
        gl::BindTexture(gl::TEXTURE_2D, texture2);

        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::REPEAT as i32);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::REPEAT as i32);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR_MIPMAP_LINEAR as i32);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32);

        let img = image::open(&Path::new("assets/textures/awesomeface.png")).expect("Failed to load texture");
        let img = img.flipv();
        let data = img.as_bytes();

        gl::TexImage2D(
            gl::TEXTURE_2D,
            0,
            gl::RGB as i32,
            img.width() as i32,
            img.height() as i32,
            0,
            gl::RGBA,
            gl::UNSIGNED_BYTE,
            data.as_ptr() as *const gl::types::GLvoid
        );
        gl::GenerateMipmap(gl::TEXTURE_2D);

        shader_program.use_program();

        let (texture1_cstr, texture2_cstr) = (
            &CString::new("texture1").unwrap(),
            &CString::new("texture2").unwrap()
        );
        shader_program.set_int(&texture1_cstr, 0);
        shader_program.set_int(&texture2_cstr, 1);


        (shader_program, vao, vbo, ebo, texture1, texture2)
    };

    // Render loop, each iteration is a "frame"
    while !window.should_close() {
        process_events(&mut window, &events);

        unsafe {
            shader_program.use_program();

            gl::ClearColor(1.0, 0.0, 1.0, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT);

            // bind textures on corresponding texture units
            gl::ActiveTexture(gl::TEXTURE0);
            gl::BindTexture(gl::TEXTURE_2D, texture1);
            gl::ActiveTexture(gl::TEXTURE1);
            gl::BindTexture(gl::TEXTURE_2D, texture2);

            gl::BindVertexArray(vao);
            // gl::DrawArrays(gl::TRIANGLES, 0, 3);
            gl::DrawElements(gl::TRIANGLES, 6, gl::UNSIGNED_INT, std::ptr::null());
            gl::BindVertexArray(0);
        }

        window.swap_buffers();
        glfw.poll_events();
    }

    // Delete GL objects
    unsafe {
        gl::DeleteVertexArrays(1, &vao);
        gl::DeleteBuffers(1, &vbo);
        gl::DeleteBuffers(1, &ebo);
    }
}

fn process_events(window: &mut glfw::Window, events: &Receiver<(f64, glfw::WindowEvent)>) {
    for (_, event) in glfw::flush_messages(events) {
        match event {
            glfw::WindowEvent::FramebufferSize(width, height) => {
                unsafe { gl::Viewport(0, 0, width, height) }
            }
            glfw::WindowEvent::Key(Key::Escape, _, Action::Press, _) => window.set_should_close(true),
            _ => {}
        }
    }
}