extern crate gl;
extern crate glfw;
extern crate image;
extern crate rand;

mod shader_program;
mod camera;
mod mesh;
mod model;
mod framebuffer;
mod skybox;
mod uniform_buffer;
mod light;

use self::glfw::{Context, Key, Action};
use std::sync::mpsc::Receiver;
use camera::{Camera, CameraMovement};
use light::{DirLight, PointLight};
use shader_program::ShaderProgram;
use framebuffer::Framebuffer;
use cgmath::{prelude::*, vec3,  Deg, Point3, Matrix4};
use skybox::Skybox;
use uniform_buffer::UniformBuffer;
use self::rand::Rng;

const MSAA: u32 = 4;

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
    camera.position = Point3 { x: 0.0, y: 10.0, z: 60.0 };

    let mut glfw: glfw::Glfw = glfw::init(glfw::FAIL_ON_ERRORS).unwrap();
    glfw.window_hint(glfw::WindowHint::ContextVersion(3, 3));
    glfw.window_hint(glfw::WindowHint::OpenGlProfile(glfw::OpenGlProfileHint::Core));
    // glfw.window_hint(glfw::WindowHint::Samples(Some(MSAA))); // Useless since using custom framebuffer
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

    let (
        shader_program,
        framebuffer_shader_program,
        skybox_shader_program,
        mut framebuffer,
        planet_model,
        rock_model,
        backpack_model,
        skybox,
        uniform_buffer
    ) = unsafe {
        // Get transforms for all the asteroids and the planet
        let mut rock_model_transforms: Vec<Matrix4<f32>> = vec![];
        let mut planet_model_transform = Matrix4::<f32>::from_translation(vec3(0.0, 0.0, 0.0));
        let mut backpack_model_transform = Matrix4::<f32>::from_translation(vec3(0.0, 10.0, 57.0));
        let mut floor_model_transform = Matrix4::<f32>::from_translation(vec3(0.0, 9.5, 60.0));
        let amount: u32 = 1_000;
        let mut rng = rand::thread_rng();
        let radius: f32 = 30.0;
        let offset: f32 = 5.0;

        planet_model_transform = planet_model_transform * Matrix4::from_scale(4.0);
        backpack_model_transform = backpack_model_transform * Matrix4::from_scale(0.2);
        floor_model_transform = floor_model_transform * Matrix4::from_nonuniform_scale(6.0, 0.01, 6.0);
        floor_model_transform = floor_model_transform * Matrix4::from_angle_x(Deg(90.0));
        
        for i in 0..amount {
            let angle = i as f32 / amount as f32 * 360.0;
            let mut displacement = (rng.gen::<i32>() % (2.0 * offset * 100.0) as i32) as f32 / 100.0 - offset;
            let x = angle.sin() * radius + displacement;
            displacement = (rng.gen::<i32>() % (2.0 * offset * 100.0) as i32) as f32 / 100.0 - offset;
            let y = displacement * 0.4; // Keep height of asteroid field smaller compared to width of x and z
            displacement = (rng.gen::<i32>() % (2.0 * offset * 100.0) as i32) as f32 / 100.0 - offset;
            let z = angle.cos() * radius + displacement;
            let mut model_transform = Matrix4::<f32>::from_translation(vec3(x, y, z));

            // Scale between 0.05 and 0.25
            let scale = (rng.gen::<i32>() % 20) as f32 / 100.0 + 0.05;
            model_transform = model_transform * Matrix4::from_scale(scale);

            // Add random rotation around a semi randomly picked rotation axis vector
            let rot_angle = (rng.gen::<i32>() % 360) as f32;
            model_transform = model_transform * Matrix4::from_axis_angle(vec3(0.4, 0.6, 0.8).normalize(), Deg(rot_angle));

            rock_model_transforms.push(model_transform);
        }

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
            height,
            MSAA
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

        // Enable multisampling
        // gl::Enable(gl::MULTISAMPLE);

        let planet_model = model::Model::new(
            "assets/models/planet/planet.obj",
            vec![planet_model_transform]
        );
        let rock_model = model::Model::new(
            "assets/models/rock/rock.obj",
            rock_model_transforms
        );
        let backpack_model = model::Model::new(
            "assets/models/backpack/backpack.obj",
            vec![backpack_model_transform, floor_model_transform]
        );

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
            planet_model,
            rock_model,
            backpack_model,
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

    let dir_light = DirLight {
        direction: vec3(-0.2, -1.0, -0.3),
        ambient: vec3(0.05, 0.05, 0.05),
        diffuse: vec3(1.0, 1.0, 1.0),
        specular: vec3(0.5, 0.5, 0.5)
    };
    let point_light = PointLight {
        position: vec3(0.7, 10.2, 59.0),
        ambient: vec3(0.00, 0.00, 0.00),
        diffuse: vec3(0.8, 0.8, 0.8),
        specular: vec3(1.0, 1.0, 1.0),
        constant: 1.0,
        linear: 0.09,
        quadratic: 0.032,
        array_position: 0
    };

    unsafe {
        // Use needs to be called before setting these even if you have the location
        shader_program.use_program();
        shader_program.set_float("material.shininess", 32.0);

        // Send light data to shader
        dir_light.send_data(&shader_program);
        point_light.send_data(&shader_program);

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
            gl::ClearColor(0.0, 0.0, 0.0, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);

            let view_transform = camera.get_view_matrix();

            uniform_buffer.write_data::<Matrix4<f32>>(
                view_transform.as_ptr() as *const gl::types::GLvoid,
                std::mem::size_of::<Matrix4<f32>>() as u32
            );

            shader_program.use_program();
            shader_program.set_vector_3("viewPos", &camera.position.to_vec());

            // START - DRAW MODELS HERE

            // Draw planet model
            planet_model.draw(&shader_program);

            // Draw the rocks
            rock_model.draw(&shader_program);

            // Draw backpack for light testing
            backpack_model.draw(&shader_program);

            // END - DRAW MODELS HERE

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