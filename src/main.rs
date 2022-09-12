extern crate gl;
extern crate glfw;
extern crate image;

mod shader_program;
mod camera;
mod mesh;
mod model;
mod framebuffer;
mod skybox;
mod uniform_buffer;

use self::glfw::{Context, Key, Action};
use std::sync::mpsc::Receiver;
use camera::{Camera, CameraMovement};
use shader_program::ShaderProgram;
use framebuffer::Framebuffer;
use cgmath::{prelude::*, vec3,  Rad, Deg, Point3, Matrix4};
use skybox::Skybox;
use uniform_buffer::UniformBuffer;

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

    let model_positions: [cgmath::Vector3<f32>; 10] = [
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

    let (
        shader_program,
        framebuffer_shader_program,
        skybox_shader_program,
        mut framebuffer,
        model,
        skybox,
        uniform_buffer
    ) = unsafe {
        let shader_program = ShaderProgram::new(
            "assets/shaders/shader.vert",
            "assets/shaders/shader.frag",
            Some("assets/shaders/shader.geom")
        );
        let framebuffer_shader_program = ShaderProgram::new(
            "assets/shaders/framebuffer.vert",
            "assets/shaders/framebuffer.frag",
            None
        );
        let skybox_shader_program = ShaderProgram::new(
            "assets/shaders/skybox.vert",
            "assets/shaders/skybox.frag",
            None
        );

        let framebuffer = Framebuffer::new(
            width,
            height
        );

        // Set this as the rendered framebuffer, it then handles switching
        framebuffer.bind_buffer();

        // Depth testing
        gl::Enable(gl::DEPTH_TEST);
        gl::DepthFunc(gl::LESS);

        // Blending
        gl::Enable(gl::BLEND);
        gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);

        // Face culling
        gl::Enable(gl::CULL_FACE);

        let model = model::Model::new("assets/models/backpack/backpack.obj");

        let skybox = Skybox::new(vec![
            "assets/textures/skybox/right.jpg".to_owned(),
            "assets/textures/skybox/left.jpg".to_owned(),
            "assets/textures/skybox/top.jpg".to_owned(),
            "assets/textures/skybox/bottom.jpg".to_owned(),
            "assets/textures/skybox/front.jpg".to_owned(),
            "assets/textures/skybox/back.jpg".to_owned()
        ]);

        let uniform_buffer = UniformBuffer::new(
            &[&shader_program, &skybox_shader_program],
            "Matrices",
            2 * std::mem::size_of::<Matrix4<f32>>() as u32
        );

        // Draw in wireframe
        // gl::PolygonMode(gl::FRONT_AND_BACK, gl::LINE);

        (
            shader_program,
            framebuffer_shader_program,
            skybox_shader_program,
            framebuffer,
            model,
            skybox,
            uniform_buffer
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
        shader_program.set_float("material.shininess", 32.0);

        // directional light
        shader_program.set_vec3("dirLight.direction", -0.2, -1.0, -0.3);
        shader_program.set_vec3("dirLight.ambient", 0.05, 0.05, 0.05);
        shader_program.set_vec3("dirLight.diffuse", 0.4, 0.4, 0.4);
        shader_program.set_vec3("dirLight.specular", 0.5, 0.5, 0.5);

        // Set projection for all shaders that require it
        uniform_buffer.write_data::<Matrix4<f32>>(projection_transform.as_ptr() as *const gl::types::GLvoid, 0);
    }

    // Render loop, each iteration is a "frame"
    while !window.should_close() {
        let current_frame = glfw.get_time() as f32;
        delta_time = current_frame - last_frame;
        last_frame = current_frame;

        process_events(
            &mut window,
            &events,
            &delta_time,
            &mut last_x,
            &mut last_y,
            &mut first_mouse,
            &mut camera,
            &mut width,
            &mut height,
            &mut framebuffer,
            &uniform_buffer
        );

        unsafe {
            framebuffer.bind_buffer(); // Buffer is set to default later so it can be rendered

            // Colour buffer does not need to be cleared when skybox is active
            gl::Clear(gl::DEPTH_BUFFER_BIT);

            let view_transform = camera.get_view_matrix();

            uniform_buffer.write_data::<Matrix4<f32>>(
                view_transform.as_ptr() as *const gl::types::GLvoid,
                std::mem::size_of::<Matrix4<f32>>() as u32
            );

            shader_program.use_program();
            shader_program.set_vector_3("viewPos", &camera.position.to_vec());

            let mut model_transforms: Vec<Matrix4<f32>> = vec![];

            for (i, position) in model_positions.iter().enumerate() {
                let mut model_transform = cgmath::Matrix4::from_translation(*position);
                let angle = current_frame * i as f32;
                model_transform = model_transform * cgmath::Matrix4::from_axis_angle(vec3(1.0, 0.3, 0.5).normalize(), Rad(angle));
                model_transform = model_transform * Matrix4::from_scale(0.2); // Smallify

                model_transforms.push(model_transform);

                shader_program.use_program();
                shader_program.set_mat4(format!("models[{}]", i).as_str(), &model_transform);
            }

            model.draw_instanced(&shader_program, model_transforms.len() as i32);

            // Drawn last so it only is drawn over unused pixels, improving performance
            skybox.draw(&skybox_shader_program);

            // Draw framebuffer
            framebuffer.draw(&framebuffer_shader_program);
        }

        window.swap_buffers();
        glfw.poll_events();
    }

    // TODO: Delete GL objects when they exit scope
}

fn process_events(
    window: &mut glfw::Window,
    events: &Receiver<(f64, glfw::WindowEvent)>,
    delta_time: &f32,
    last_x: &mut f32,
    last_y: &mut f32,
    first_mouse: &mut bool,
    camera: &mut Camera,
    width: &mut u32,
    height: &mut u32,
    framebuffer: &mut Framebuffer,
    uniform_buffer: &UniformBuffer
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
                    
                    uniform_buffer.write_data::<Matrix4<f32>>(projection_transform.as_ptr() as *const gl::types::GLvoid, 0);

                    framebuffer.resize(*width, *height);
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
                    uniform_buffer.write_data::<Matrix4<f32>>(projection_transform.as_ptr() as *const gl::types::GLvoid, 0);
                }
            }
            _ => {}
        }
    }

    if window.get_key(Key::W) == Action::Press {
        camera.process_keyboard(CameraMovement::FORWARD, *delta_time);
    }
    if window.get_key(Key::S) == Action::Press {
        camera.process_keyboard(CameraMovement::BACKWARD, *delta_time);
    }
    if window.get_key(Key::A) == Action::Press {
        camera.process_keyboard(CameraMovement::LEFT, *delta_time);
    }
    if window.get_key(Key::D) == Action::Press {
        camera.process_keyboard(CameraMovement::RIGHT, *delta_time);
    }
}