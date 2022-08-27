extern crate gl;
extern crate glfw;
extern crate image;

mod shader_program;
mod camera;

use self::glfw::{Context, Key, Action};
use std::{sync::mpsc::Receiver, vec};
use camera::{Camera, CameraMovement};
use shader_program::ShaderProgram;
use std::path::Path;
use std::ffi::CString;
use cgmath::{prelude::*, vec3,  Rad, Deg, Point3, Matrix4};

fn main() {
    let mut width = 800;
    let mut height = 600;

    // Timing
    let mut delta_time: f32; // Time between current frame and last frame
    let mut last_frame: f32 = 0.0;

    let mut last_x = width as f32 / 2.0;
    let mut last_y = height as f32 / 2.0;

    let mut first_mouse = true;

    let mut camera = Camera::default();
    camera.position = Point3 { x: 0.0, y: 0.0, z: 3.0 };

    let light_pos = vec3(1.2, 1.0, 2.0);
    // let light_pos = vec3(0.0, 0.0, 0.0);

    let mut glfw: glfw::Glfw = glfw::init(glfw::FAIL_ON_ERRORS).unwrap();
    glfw.window_hint(glfw::WindowHint::ContextVersion(3, 3));
    glfw.window_hint(glfw::WindowHint::OpenGlProfile(glfw::OpenGlProfileHint::Core));
    #[cfg(target_os = "macos")] glfw.window_hint(glfw::WindowHint::OpenGlForwardCompat(true));
    

    let (mut window, events) = glfw.create_window(
        width,
        height,
        "LearnOpenGL",
        glfw::WindowMode::Windowed
    ).expect("Failed to create GLFW window");

    window.make_current();
    window.set_key_polling(true);
    window.set_framebuffer_size_polling(true);
    window.set_cursor_pos_polling(true);
    window.set_scroll_polling(true);
    window.set_cursor_mode(glfw::CursorMode::Disabled);

    gl::load_with(|symbol| window.get_proc_address(symbol) as *const _);
    
    // Triangle
    let vertices: Vec<f32> = vec![
        -0.5, -0.5, -0.5,  0.0, 0.0, 0.0,  0.0, -1.0,
        0.5, -0.5, -0.5,  1.0, 0.0, 0.0,  0.0, -1.0,
        0.5,  0.5, -0.5,  1.0, 1.0, 0.0,  0.0, -1.0,
        0.5,  0.5, -0.5,  1.0, 1.0, 0.0,  0.0, -1.0,
        -0.5,  0.5, -0.5,  0.0, 1.0, 0.0,  0.0, -1.0,
        -0.5, -0.5, -0.5,  0.0, 0.0, 0.0,  0.0, -1.0,

        -0.5, -0.5,  0.5,  0.0, 0.0, 0.0,  0.0, 1.0,
        0.5, -0.5,  0.5,  1.0, 0.0, 0.0,  0.0, 1.0,
        0.5,  0.5,  0.5,  1.0, 1.0, 0.0,  0.0, 1.0,
        0.5,  0.5,  0.5,  1.0, 1.0, 0.0,  0.0, 1.0,
        -0.5,  0.5,  0.5,  0.0, 1.0, 0.0,  0.0, 1.0,
        -0.5, -0.5,  0.5,  0.0, 0.0, 0.0,  0.0, 1.0,

        -0.5,  0.5,  0.5,  1.0, 0.0, -1.0,  0.0,  0.0,
        -0.5,  0.5, -0.5,  1.0, 1.0, -1.0,  0.0,  0.0,
        -0.5, -0.5, -0.5,  0.0, 1.0, -1.0,  0.0,  0.0,
        -0.5, -0.5, -0.5,  0.0, 1.0, -1.0,  0.0,  0.0,
        -0.5, -0.5,  0.5,  0.0, 0.0, -1.0,  0.0,  0.0,
        -0.5,  0.5,  0.5,  1.0, 0.0, -1.0,  0.0,  0.0,

        0.5,  0.5,  0.5,  1.0, 0.0, 1.0,  0.0,  0.0,
        0.5,  0.5, -0.5,  1.0, 1.0, 1.0,  0.0,  0.0,
        0.5, -0.5, -0.5,  0.0, 1.0, 1.0,  0.0,  0.0,
        0.5, -0.5, -0.5,  0.0, 1.0, 1.0,  0.0,  0.0,
        0.5, -0.5,  0.5,  0.0, 0.0, 1.0,  0.0,  0.0,
        0.5,  0.5,  0.5,  1.0, 0.0, 1.0,  0.0,  0.0,

        -0.5, -0.5, -0.5,  0.0, 1.0, 0.0, -1.0,  0.0,
        0.5, -0.5, -0.5,  1.0, 1.0, 0.0, -1.0,  0.0,
        0.5, -0.5,  0.5,  1.0, 0.0, 0.0, -1.0,  0.0,
        0.5, -0.5,  0.5,  1.0, 0.0, 0.0, -1.0,  0.0,
        -0.5, -0.5,  0.5,  0.0, 0.0, 0.0, -1.0,  0.0,
        -0.5, -0.5, -0.5,  0.0, 1.0, 0.0, -1.0,  0.0,

        -0.5,  0.5, -0.5,  0.0, 1.0, 0.0,  1.0,  0.0,
        0.5,  0.5, -0.5,  1.0, 1.0, 0.0,  1.0,  0.0,
        0.5,  0.5,  0.5,  1.0, 0.0, 0.0,  1.0,  0.0,
        0.5,  0.5,  0.5,  1.0, 0.0, 0.0,  1.0,  0.0,
        -0.5,  0.5,  0.5,  0.0, 0.0, 0.0,  1.0,  0.0,
        -0.5,  0.5, -0.5,  0.0, 1.0, 0.0,  1.0,  0.0,
    ];

    let cube_positions: [cgmath::Vector3<f32>; 10] = [
        vec3(0.0, 0.0, 0.0),
        vec3(2.0, 5.0, -15.0),
        vec3(-1.5, -2.2, -2.5),
        vec3(-3.8, -2.0, -12.3),
        vec3(2.4, -0.4, -3.5),
        vec3(-1.7, 3.0, -7.5),
        vec3(1.3, -2.0, -2.5),
        vec3(1.5, 2.0, -2.5),
        vec3(1.5, 0.2, -1.5),
        vec3(-1.3, 1.0, -1.5)
    ];

    let (shader_program, light_shader_program, vao, light_vao, vbo) = unsafe {
        let shader_program = ShaderProgram::new(
            "assets/shaders/shader.vert",
            "assets/shaders/shader.frag"
        );
        let light_shader_program = ShaderProgram::new(
            "assets/shaders/shader.vert",
            "assets/shaders/light_source.frag"
        );

        gl::Enable(gl::DEPTH_TEST);

        let (mut vao, mut vbo) = (0, 0);
        gl::GenVertexArrays(1, &mut vao);
        gl::GenBuffers(1, &mut vbo);

        gl::BindVertexArray(vao);

        gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
        gl::BufferData(
            gl::ARRAY_BUFFER,
            (vertices.len() * std::mem::size_of::<f32>()) as gl::types::GLsizeiptr,
            vertices.as_ptr() as *const gl::types::GLvoid,
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

        // Texture coords
        gl::VertexAttribPointer(
            1,
            2,
            gl::FLOAT,
            gl::FALSE,
            stride,
            (3 * std::mem::size_of::<f32>()) as *const gl::types::GLvoid
        );
        gl::EnableVertexAttribArray(1);

        // Normal vectors (gosh it's getting crowded in here)
        gl::VertexAttribPointer(
            2,
            3,
            gl::FLOAT,
            gl::FALSE,
            stride,
            (5 * std::mem::size_of::<f32>()) as *const gl::types::GLvoid
        );
        gl::EnableVertexAttribArray(2);

        // Light source VAO
        let mut light_vao = 0;
        gl::GenVertexArrays(1, &mut light_vao);
        gl::BindVertexArray(light_vao);

        gl::BindBuffer(gl::ARRAY_BUFFER, vbo);

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

        // Texture coords
        // gl::VertexAttribPointer(
        //     1,
        //     2,
        //     gl::FLOAT,
        //     gl::FALSE,
        //     stride,
        //     (3 * std::mem::size_of::<f32>()) as *const gl::types::GLvoid
        // );
        // gl::EnableVertexAttribArray(1);

        // // Normal vectors (gosh it's getting crowded in here)
        // gl::VertexAttribPointer(
        //     2,
        //     3,
        //     gl::FLOAT,
        //     gl::FALSE,
        //     stride,
        //     (5 * std::mem::size_of::<f32>()) as *const gl::types::GLvoid
        // );
        // gl::EnableVertexAttribArray(2);

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

        // bind textures on corresponding texture units
        gl::ActiveTexture(gl::TEXTURE0);
        gl::BindTexture(gl::TEXTURE_2D, texture1);
        gl::ActiveTexture(gl::TEXTURE1);
        gl::BindTexture(gl::TEXTURE_2D, texture2);

        (shader_program, light_shader_program, vao, light_vao, vbo)
    };

    let (model_cstr, view_cstr, projection_cstr, light_color_cstr, light_pos_cstr) = (
        &CString::new("model").unwrap(),
        &CString::new("view").unwrap(),
        &CString::new("projection").unwrap(),
        &CString::new("lightColor").unwrap(),
        &CString::new("lightPos").unwrap()
    );

    let (model_location, view_location, projection_location, light_color_location, light_pos_location) = unsafe {
        (
            gl::GetUniformLocation(shader_program.id, model_cstr.as_ptr()),
            gl::GetUniformLocation(shader_program.id, view_cstr.as_ptr()),
            gl::GetUniformLocation(shader_program.id, projection_cstr.as_ptr()),
            gl::GetUniformLocation(shader_program.id, light_color_cstr.as_ptr()),
            gl::GetUniformLocation(shader_program.id, light_pos_cstr.as_ptr())
        )
    };

    let (light_model_location, light_view_location, light_projection_location) = unsafe {
        (
            gl::GetUniformLocation(light_shader_program.id, model_cstr.as_ptr()),
            gl::GetUniformLocation(light_shader_program.id, view_cstr.as_ptr()),
            gl::GetUniformLocation(light_shader_program.id, projection_cstr.as_ptr())
        )
    };

    let projection_transform = cgmath::perspective(
        Deg(45.0),
        width as f32 / height as f32,
        0.1,
        100.0
    );
    unsafe {
        // Use needs to be called before setting these even if you have the location
        shader_program.use_program();
        gl::UniformMatrix4fv(projection_location, 1, gl::FALSE, projection_transform.as_ptr());
        gl::Uniform3f(light_color_location, 1.0, 1.0, 1.0);
        gl::Uniform3fv(light_pos_location, 1, light_pos.as_ptr());
        light_shader_program.use_program();
        gl::UniformMatrix4fv(light_projection_location, 1, gl::FALSE, projection_transform.as_ptr());
    }

    // Render loop, each iteration is a "frame"
    while !window.should_close() {
        let current_frame = glfw.get_time() as f32;
        delta_time = current_frame - last_frame;
        last_frame = current_frame;

        process_events(
            &mut window,
            &events,
            projection_location,
            light_projection_location,
            delta_time,
            &mut last_x,
            &mut last_y,
            &mut first_mouse,
            &mut camera,
            &mut width,
            &mut height
        );

        unsafe {
            gl::ClearColor(0.1, 0.1, 0.1, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);

            shader_program.use_program();

            let view_transform = camera.get_view_matrix();

            gl::UniformMatrix4fv(view_location, 1, gl::FALSE, view_transform.as_ptr());

            gl::BindVertexArray(vao);
            for (i, position) in cube_positions.iter().enumerate() {
                let mut model_transform = cgmath::Matrix4::from_translation(*position);
                let angle = glfw.get_time() as f32 * i as f32;
                model_transform = model_transform * cgmath::Matrix4::from_axis_angle(vec3(1.0, 0.3, 0.5).normalize(), Rad(angle));

                gl::UniformMatrix4fv(model_location, 1, gl::FALSE, model_transform.as_ptr());

                gl::DrawArrays(gl::TRIANGLES, 0, 36);
            }

            light_shader_program.use_program();
            gl::UniformMatrix4fv(light_view_location, 1, gl::FALSE, view_transform.as_ptr());

            let mut model_transform = Matrix4::from_translation(light_pos);
            model_transform = model_transform * Matrix4::from_scale(0.2); // Smallify the cube
            gl::UniformMatrix4fv(light_model_location, 1, gl::FALSE, model_transform.as_ptr());

            gl::BindVertexArray(light_vao);
            gl::DrawArrays(gl::TRIANGLES, 0, 36);
        }

        window.swap_buffers();
        glfw.poll_events();
    }

    // Delete GL objects
    unsafe {
        gl::DeleteVertexArrays(1, &vao);
        gl::DeleteBuffers(1, &vbo);
    }
}

fn process_events(
    window: &mut glfw::Window,
    events: &Receiver<(f64, glfw::WindowEvent)>,
    projection_location: i32,
    light_projection_location: i32,
    delta_time: f32,
    last_x: &mut f32,
    last_y: &mut f32,
    first_mouse: &mut bool,
    camera: &mut Camera,
    width: &mut u32,
    height: &mut u32
) {
    for (_, event) in glfw::flush_messages(events) {
        match event {
            glfw::WindowEvent::FramebufferSize(window_width, window_height) => {
                *width = window_width as u32;
                *height = window_height as u32;

                unsafe {
                    gl::Viewport(0, 0, window_width, window_height);
                    let projection_transform = cgmath::perspective(
                        Deg(camera.zoom),
                        *width as f32 / *height as f32,
                        0.1,
                        100.0
                    );
                    gl::UniformMatrix4fv(projection_location, 1, gl::FALSE, projection_transform.as_ptr());
                }
            }
            glfw::WindowEvent::Key(Key::Escape, _, Action::Press, _) => window.set_should_close(true),
            glfw::WindowEvent::CursorPos(x, y) => {
                if *first_mouse {
                    *last_x = x as f32;
                    *last_y = y as f32;
                    *first_mouse = false;
                }

                let x_offset = x as f32 - *last_x;
                let y_offset = *last_y - y as f32;

                *last_x = x as f32;
                *last_y = y as f32;

                camera.process_mouse_movement(x_offset, y_offset, true);
            }
            glfw::WindowEvent::Scroll(_x_offset, y_offset) => {
                camera.process_mouse_scroll(y_offset as f32);

                let projection_transform = cgmath::perspective(
                    Deg(camera.zoom),
                    *width as f32 / *height as f32,
                    0.1,
                    100.0
                );

                unsafe {
                    gl::UniformMatrix4fv(projection_location, 1, gl::FALSE, projection_transform.as_ptr());
                    gl::UniformMatrix4fv(light_projection_location, 1, gl::FALSE, projection_transform.as_ptr());
                }
            }
            _ => {}
        }
    }

    if window.get_key(Key::W) == Action::Press {
        camera.process_keyboard(CameraMovement::FORWARD, delta_time);
    }
    if window.get_key(Key::S) == Action::Press {
        camera.process_keyboard(CameraMovement::BACKWARD, delta_time);
    }
    if window.get_key(Key::A) == Action::Press {
        camera.process_keyboard(CameraMovement::LEFT, delta_time);
    }
    if window.get_key(Key::D) == Action::Press {
        camera.process_keyboard(CameraMovement::RIGHT, delta_time);
    }
}