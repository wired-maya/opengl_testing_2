extern crate gl;
extern crate glfw;

mod shader_program;

use self::glfw::{Context, Key, Action};
use std::sync::mpsc::Receiver;
use shader_program::ShaderProgram;

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
        0.5, -0.5, 0.0,  1.0, 0.0, 0.0,  // bottom right
        -0.5, -0.5, 0.0,  0.0, 1.0, 0.0,  // bottom left
        0.0,  0.5, 0.0,  0.0, 0.0, 1.0   // top
    ];

    let (shader_program, vao) = unsafe {
        let shader_program = ShaderProgram::new(
            "assets/shaders/shader.vert",
            "assets/shaders/shader.frag"
        );

        // let vertices: Vec<f32> = vec![
        //     0.5, 0.5, 0.0,  // top right
        //     0.5, -0.5, 0.0,  // bottom right
        //     -0.5, -0.5, 0.0,  // bottom left
        //     -0.5, 0.5, 0.0   // top left 
        // ];
        // let indices: Vec<u8> = vec![
        //     0, 1, 3,   // first triangle
        //     1, 2, 3    // second triangle
        // ];

        let (mut vao, mut vbo, mut _ebo) = (0, 0, 0);
        gl::GenVertexArrays(1, &mut vao);
        gl::GenBuffers(1, &mut vbo);
        // gl::GenBuffers(1, &mut ebo);

        gl::BindVertexArray(vao);
        gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
        // gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, ebo);

        gl::BufferData(
            gl::ARRAY_BUFFER,
            (vertices.len() * std::mem::size_of::<f32>()) as gl::types::GLsizeiptr,
            vertices.as_ptr() as *const gl::types::GLvoid,
            gl::STATIC_DRAW
        );
        // gl::BufferData(
        //     gl::ELEMENT_ARRAY_BUFFER,
        //     (vertices.len() * std::mem::size_of::<u8>()) as gl::types::GLsizeiptr,
        //     indices.as_ptr() as *const gl::types::GLvoid,
        //     gl::STATIC_DRAW
        // );

        gl::VertexAttribPointer(
            0,
            3,
            gl::FLOAT,
            gl::FALSE,
            (6 * std::mem::size_of::<f32>()) as gl::types::GLint,
            std::ptr::null()
        );
        gl::EnableVertexAttribArray(0);

        gl::VertexAttribPointer(
            1,
            3,
            gl::FLOAT,
            gl::FALSE,
            (6 * std::mem::size_of::<f32>()) as gl::types::GLint,
            (3 * std::mem::size_of::<f32>()) as *const gl::types::GLvoid
        );
        gl::EnableVertexAttribArray(1);

        gl::BindBuffer(gl::ARRAY_BUFFER, 0);
        gl::BindVertexArray(0);

        // Uncomment to enable wireframe mode
        // gl::PolygonMode(gl::FRONT_AND_BACK, gl::LINE);

        (shader_program, vao)
    };

    // Render loop, each iteration is a "frame"
    while !window.should_close() {
        process_events(&mut window, &events);

        unsafe {
            // let time_value: f64 = glfw::Glfw::get_time(&glfw);
            // let green_value: f64 = (time_value.sin() / 2.0) + 0.5;
            // let our_color = CString::new("ourColor").unwrap();
            // let vertex_color_location = gl::GetUniformLocation(shader_program, our_color.as_ptr());

            shader_program.use_program();

            // gl::Uniform4f(vertex_color_location, 0.0, green_value as f32, 0.0, 1.0);

            gl::ClearColor(1.0, 0.0, 1.0, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT);

            gl::BindVertexArray(vao);
            // gl::DrawElements(gl::TRIANGLES, 6, gl::UNSIGNED_BYTE, ptr::null());
            gl::DrawArrays(gl::TRIANGLES, 0, 3);
            gl::BindVertexArray(0);
        }

        window.swap_buffers();
        glfw.poll_events();
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