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
use std::{sync::mpsc::Receiver, ffi::{c_void, CString}, slice};
use camera::{Camera, CameraMovement};
use light::{DirLight, PointLight};
use shader_program::ShaderProgram;
use framebuffer::Framebuffer;
use cgmath::{prelude::*, vec3,  Deg, Point3, Matrix4};
use skybox::Skybox;
use uniform_buffer::UniformBuffer;
use self::rand::Rng;

const WIDTH: u32 = 800;
const HEIGHT: u32 = 600;
const MSAA: u32 = 4;
const SHADOW_RES: u32 = 1024;

fn main() {
    let mut width = WIDTH;
    let mut height = HEIGHT;

    // Timing
    let mut delta_time: f32; // Time between current frame and last frame
    let mut last_frame: f32 = 0.0;

    let mut last_x = width as f32 / 2.0;
    let mut last_y = height as f32 / 2.0;

    let mut first_mouse = true;

    let mut camera = Camera::default();
    camera.position = Point3 { x: 0.0, y: 00.0, z: 30.0 };

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

    // Set all OpenGL parameters
    unsafe {
        // Create GL context
        gl::load_with(|symbol| window.get_proc_address(symbol) as *const _);

        // Depth testing
        gl::Enable(gl::DEPTH_TEST);
        gl::DepthFunc(gl::LESS);

        // Blending
        gl::Enable(gl::BLEND);
        gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);

        // Face culling
        gl::Enable(gl::CULL_FACE);

        // Enable debug with callback for simple error printing
        gl::Enable(gl::DEBUG_OUTPUT);
        gl::DebugMessageCallback(
            Some(debug_message_callback),
            std::ptr::null()
        );

        // Enable multisampling
        // gl::Enable(gl::MULTISAMPLE);

        // Draw in wireframe
        // gl::PolygonMode(gl::FRONT_AND_BACK, gl::LINE);
    }

    // Get transforms for all the asteroids and the planet
    let mut rock_model_transforms: Vec<Matrix4<f32>> = vec![];
    let mut planet_model_transform = Matrix4::<f32>::from_translation(vec3(0.0, 0.0, 0.0));
    let mut backpack_model_transform = Matrix4::<f32>::from_translation(vec3(15.0, -23.0, 0.0));
    let mut floor_model_transform = Matrix4::<f32>::from_translation(vec3(0.0, -25.0, 0.0));
    let amount: u32 = 1_000;
    let mut rng = rand::thread_rng();
    let radius: f32 = 30.0;
    let offset: f32 = 5.0;

    planet_model_transform = planet_model_transform * Matrix4::from_scale(4.0);
    backpack_model_transform = backpack_model_transform * Matrix4::from_scale(10.0);
    floor_model_transform = floor_model_transform * Matrix4::from_nonuniform_scale(36.0, 1.0, 36.0);
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

    let mut shader_program = ShaderProgram::new(
        "assets/shaders/shader.vert".to_string(),
        "assets/shaders/shader.frag".to_string(),
        Some("assets/shaders/shader.geom".to_string())
    );
    let mut framebuffer_shader_program = ShaderProgram::new(
        "assets/shaders/framebuffer.vert".to_string(),
        "assets/shaders/framebuffer.frag".to_string(),
        None
    );
    let mut skybox_shader_program = ShaderProgram::new(
        "assets/shaders/skybox.vert".to_string(),
        "assets/shaders/skybox.frag".to_string(),
        None
    );
    let mut depth_shader_program = ShaderProgram::new(
        "assets/shaders/depth.vert".to_string(),
        "assets/shaders/depth.frag".to_string(),
        None
    );
    let mut cube_depth_shader_program = ShaderProgram::new(
        "assets/shaders/cube_depth_shader.vert".to_string(),
        "assets/shaders/cube_depth_shader.frag".to_string(),
        Some("assets/shaders/cube_depth_shader.geom".to_string())
    );
    let mut debug_shader_program = ShaderProgram::new(
        "assets/shaders/debug_shader.vert".to_string(),
        "assets/shaders/debug_shader.frag".to_string(),
        Some("assets/shaders/debug_shader.geom".to_string())
    );

    let mut framebuffer = Framebuffer::new(
        width,
        height,
        MSAA
    );

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
        vec![
            backpack_model_transform,
            floor_model_transform
        ]
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
        &[&shader_program, &skybox_shader_program, &debug_shader_program],
        "Matrices",
        2 * std::mem::size_of::<Matrix4<f32>>() as u32
    );

    let mut projection_transform = cgmath::perspective(
        Deg(45.0),
        width as f32 / height as f32,
        0.1,
        500.0
    );

    let dir_light = DirLight::new(
        vec3(0.0, 40.0, 40.0),
        vec3(0.05, 0.05, 0.05),
        vec3(1.0, 1.0, 1.0),
        vec3(0.5, 0.5, 0.5),
        SHADOW_RES
    );
    let point_light = PointLight::new(
        vec3(20.0, -15.0, 0.0),
        vec3(0.05, 0.05, 0.05),
        vec3(0.8, 0.8, 0.8),
        vec3(1.0, 1.0, 1.0),
        1.0,
        0.007,
        0.0002,
        0,
        SHADOW_RES
    );

    let mut should_resend_data = true;
    let mut show_debug = false;

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
            &uniform_buffer,
            &mut [
                &mut shader_program,
                &mut framebuffer_shader_program,
                &mut skybox_shader_program,
                &mut depth_shader_program,
                &mut cube_depth_shader_program,
                &mut debug_shader_program
            ],
            &mut should_resend_data,
            &mut projection_transform,
            &mut show_debug
        );

        if should_resend_data {
            unsafe {
                // Set this as the rendered framebuffer, it then handles switching
                framebuffer.bind_buffer();

                // Use needs to be called before setting these even if you have the location
                shader_program.use_program();
                shader_program.set_float("material.shininess", 32.0);

                // Send light data to shader
                dir_light.send_lighting_data(&shader_program);
                point_light.send_lighting_data(&shader_program);
                shader_program.set_float("pointLight.far_plane", 300.0); // Temp

                // Already has a use program
                // TODO: simple rule should be to call use program before you pass it anywhere,
                // TODO: therefore to reduce calls to it, remove it from struct functions and
                // TODO: make it manual
                // TODO: Also make this a uniform buffer to reduce calls
                dir_light.configure_shader_and_matrices(&depth_shader_program);
                dir_light.configure_shader_and_matrices(&shader_program);

                point_light.configure_shader_and_matrices(&cube_depth_shader_program);

                // Set projection for all shaders that require it
                uniform_buffer.write_data::<Matrix4<f32>>(projection_transform.as_ptr() as *const gl::types::GLvoid, 0);

                should_resend_data = false;
            }
        }

        unsafe {
            let view_transform = camera.get_view_matrix();

            uniform_buffer.write_data::<Matrix4<f32>>(
                view_transform.as_ptr() as *const gl::types::GLvoid,
                std::mem::size_of::<Matrix4<f32>>() as u32
            );

            // START - DRAW MODELS HERE

            // Fix peter panning
            gl::CullFace(gl::FRONT);

            // Draw to depth buffer for lighting
            dir_light.bind_buffer();

            depth_shader_program.use_program();
            planet_model.draw(&depth_shader_program);
            rock_model.draw(&depth_shader_program);

            // Reset
            gl::CullFace(gl::BACK);

            // Floor, doesn't cast shadows so don't cull front faces
            backpack_model.draw(&depth_shader_program);

            // Fix peter panning
            gl::CullFace(gl::FRONT);

            // Draw to depth buffer for lighting
            point_light.bind_buffer();

            cube_depth_shader_program.use_program();
            planet_model.draw(&cube_depth_shader_program);
            rock_model.draw(&cube_depth_shader_program);

            // Reset
            gl::CullFace(gl::BACK);

            // Floor, doesn't cast shadows so don't cull front faces
            backpack_model.draw(&cube_depth_shader_program);

            // Draw to regular framebuffer for an actual scene
            framebuffer.bind_buffer();

            shader_program.use_program();
            shader_program.set_vector_3("viewPos", &camera.position.to_vec());
            dir_light.bind_shadow_map(&shader_program);
            point_light.bind_shadow_map(&shader_program);
            planet_model.draw(&shader_program);
            rock_model.draw(&shader_program);
            backpack_model.draw(&shader_program);

            if show_debug {
                debug_shader_program.use_program();
                planet_model.draw(&debug_shader_program);
                rock_model.draw(&debug_shader_program);
                backpack_model.draw(&debug_shader_program);
            }

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

// Callback function intended to be called from C
extern "system" fn debug_message_callback(
    source: u32,
    type_: u32,
    _id: u32,
    severity: u32,
    length: i32,
    message: *const i8,
    _user_param: *mut c_void
) {
    let type_str = match type_ {
        gl::DEBUG_TYPE_ERROR => "ERROR",
        gl::DEBUG_TYPE_DEPRECATED_BEHAVIOR => "DEPRECATED_BEHAVIOR",
        gl::DEBUG_TYPE_UNDEFINED_BEHAVIOR => "UNDEFINED_BEHAVIOR",
        gl::DEBUG_TYPE_PORTABILITY => "PORTABILITY",
        gl::DEBUG_TYPE_PERFORMANCE => "PERFORMANCE",
        gl::DEBUG_TYPE_MARKER => "MARKER",
        gl::DEBUG_TYPE_PUSH_GROUP => "PUSH_GROUP",
        gl::DEBUG_TYPE_POP_GROUP => "POP_GROUP",
        gl::DEBUG_TYPE_OTHER => "OTHER",
        _ => "OTHER"
    };
    let source_str = match source {
        gl::DEBUG_SOURCE_API => "API",
        gl::DEBUG_SOURCE_WINDOW_SYSTEM => "WINDOW_SYSTEM",
        gl::DEBUG_SOURCE_SHADER_COMPILER => "SHADER_COMPILER",
        gl::DEBUG_SOURCE_THIRD_PARTY => "THIRD_PARTY",
        gl::DEBUG_SOURCE_APPLICATION => "APPLICATION",
        gl::DEBUG_SOURCE_OTHER => "OTHER",
        _ => "OTHER"
    };
    let severity_str = match severity {
        gl::DEBUG_SEVERITY_HIGH => "HIGH_SEVERITY",
        gl::DEBUG_SEVERITY_MEDIUM => "MEDIUM_SEVERITY",
        gl::DEBUG_SEVERITY_LOW => "LOW_SEVERITY",
        gl::DEBUG_SEVERITY_NOTIFICATION => "NOTIFICATION",
        _ => "UNKNOWN_SEVERITY"
    };
    let message_cstr = unsafe {
        let buffer = slice::from_raw_parts(message as *const u8, length as usize);
        CString::from_vec_unchecked(buffer.to_vec())
    };

    println!("{}::{}::{}::{}", type_str, source_str, severity_str, message_cstr.to_str().unwrap());
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
    uniform_buffer: &UniformBuffer,
    shader_programs: &mut [&mut ShaderProgram],
    should_resend_data: &mut bool,
    projection_transform: &mut Matrix4<f32>,
    show_debug: &mut bool
) {
    for (_, event) in glfw::flush_messages(events) {
        match event {
            glfw::WindowEvent::FramebufferSize(window_width, window_height) => {
                *width = window_width as u32;
                *height = window_height as u32;

                unsafe {
                    gl::Viewport(0, 0, window_width, window_height);
                    *projection_transform = cgmath::perspective(
                        Deg(camera.zoom),
                        *width as f32 / *height as f32,
                        0.1,
                        500.0
                    );
                    
                    uniform_buffer.write_data::<Matrix4<f32>>(projection_transform.as_ptr() as *const gl::types::GLvoid, 0);

                    framebuffer.resize(*width, *height);
                }
            }
            glfw::WindowEvent::Key(Key::Escape, _, Action::Press, _) => window.set_should_close(true),
            glfw::WindowEvent::Key(Key::R, _, Action::Press, _) => {
                for i in 0..shader_programs.len() {
                    shader_programs[i].reload();
                }

                *should_resend_data = true;
            }
            glfw::WindowEvent::Key(Key::E, _, Action::Press, _) => {
                *show_debug = !*show_debug;
            }
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

                *projection_transform = cgmath::perspective(
                    Deg(camera.zoom),
                    *width as f32 / *height as f32,
                    0.1,
                    500.0
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