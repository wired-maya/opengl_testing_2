extern crate gl;
extern crate glfw;
extern crate image;
extern crate rand;

mod gl_safe;

use self::glfw::{Context, Key, Action};
use std::{sync::mpsc::Receiver, ffi::{c_void, CString}, slice};
use gl_safe::*;
use cgmath::{prelude::*, vec3,  Deg, Point3, Matrix4, Vector3};
use self::rand::Rng;

const WIDTH: u32 = 800;
const HEIGHT: u32 = 600;
const MSAA: u32 = 4;
const _SHADOW_RES: u32 = 1024;

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
    camera.position = Point3 { x: 0.0, y: 0.0, z: 1.0 };

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
        // gl::Enable(gl::BLEND);
        // gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
        gl::Disable(gl::BLEND);

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

    let mut shader_program = ShaderProgram::new(
        "assets/shaders/shader.vert".to_string(),
        "assets/shaders/deffered_shader.frag".to_string(),
        // Some("assets/shaders/shader.geom".to_string())
        None
    ).unwrap();
    let mut framebuffer_shader_program = ShaderProgram::new(
        "assets/shaders/framebuffer.vert".to_string(),
        "assets/shaders/framebuffer.frag".to_string(),
        None
    ).unwrap();
    let mut skybox_shader_program = ShaderProgram::new(
        "assets/shaders/skybox.vert".to_string(),
        "assets/shaders/skybox.frag".to_string(),
        None
    ).unwrap();
    let mut depth_shader_program = ShaderProgram::new(
        "assets/shaders/depth.vert".to_string(),
        "assets/shaders/depth.frag".to_string(),
        None
    ).unwrap();
    let mut cube_depth_shader_program = ShaderProgram::new(
        "assets/shaders/cube_depth_shader.vert".to_string(),
        "assets/shaders/cube_depth_shader.frag".to_string(),
        Some("assets/shaders/cube_depth_shader.geom".to_string())
    ).unwrap();
    let mut debug_shader_program = ShaderProgram::new(
        "assets/shaders/debug_shader.vert".to_string(),
        "assets/shaders/debug_shader.frag".to_string(),
        Some("assets/shaders/debug_shader.geom".to_string())
    ).unwrap();
    let mut blur_shader_program = ShaderProgram::new(
        "assets/shaders/framebuffer.vert".to_string(),
        "assets/shaders/gaussian_blur.frag".to_string(),
        None
    ).unwrap();
    let mut light_shader_program = ShaderProgram::new(
        "assets/shaders/light_source.vert".to_string(),
        "assets/shaders/light_source.frag".to_string(),
        None
    ).unwrap();
    let mut lighting_pass_shader_program = ShaderProgram::new(
        "assets/shaders/framebuffer.vert".to_string(),
        "assets/shaders/lighting_pass_shader.frag".to_string(),
        None
    ).unwrap();

    let distance_scale = 2.0;

    let mut light_positions: Vec<Vector3<f32>> = vec![];
    let mut planet_transforms: Vec<Matrix4<f32>> = vec![];
    let mut light_colors: Vec<Vector3<f32>> = vec![];
    let amount = 4;

    let mut rnd = rand::thread_rng();

    // Temp light radius, this will be handled per-light later
    let constant = 1.0;
    let linear = 0.7;
    let quadratic = 1.8;
    let mut light_radii = vec![];

    for x in 0..amount {
        for z in 0..amount {
            // Maybe add random offset to create a rough surface, perfect for showing off cool light?
            let transform: Vector3<f32> = vec3(x as f32, 0.0, z as f32) * distance_scale;
            let mut matrix = Matrix4::<f32>::from_translation(transform);
            matrix = matrix * Matrix4::<f32>::from_scale(distance_scale / 10.0);
            planet_transforms.push(matrix);
            light_positions.push(transform + (vec3(0.5, 0.0, 0.5) * distance_scale));

            // let color = if x % 2 == 0 { vec3(1.0, 0.0, 0.0) } 
            // else { vec3(0.0, 1.0, 0.0) };

            let random_nums = rnd.gen::<(f32, f32, f32)>();
            let color = vec3(random_nums.0, random_nums.1, random_nums.2);

            light_colors.push(color);

            let light_max: f32 = f32::max(
                f32::max(random_nums.0, random_nums.1),
                random_nums.2
            );
            // This hurts to look at, I know...
            let radius = (-linear + f32::sqrt(
                linear * linear - 4.0 * quadratic * (constant - (256.0 / 5.0) * light_max)
            )) / (2.0 * quadratic);

            light_radii.push(radius);
        }
    }

    let mut framebuffer = RenderPipeline::new(
        width,
        height
    );

    let planet_model = Model::new(
        "assets/models/planet/planet.obj",
        planet_transforms
    ).unwrap();

    let skybox = Skybox::new(vec![
        "assets/textures/skybox/right.jpg".to_owned(),
        "assets/textures/skybox/left.jpg".to_owned(),
        "assets/textures/skybox/top.jpg".to_owned(),
        "assets/textures/skybox/bottom.jpg".to_owned(),
        "assets/textures/skybox/front.jpg".to_owned(),
        "assets/textures/skybox/back.jpg".to_owned()
    ]).unwrap();

    let uniform_buffer = UniformBuffer::new(
        &[&shader_program, &skybox_shader_program, &debug_shader_program, &light_shader_program],
        "Matrices",
        2 * std::mem::size_of::<Matrix4<f32>>() as u32
    ).unwrap();

    let mut projection_transform = cgmath::perspective(
        Deg(45.0),
        width as f32 / height as f32,
        0.1,
        500.0
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
                &mut debug_shader_program,
                &mut blur_shader_program,
                &mut light_shader_program,
                &mut lighting_pass_shader_program
            ],
            &mut should_resend_data,
            &mut projection_transform,
            &mut show_debug
        );

        if should_resend_data {
            unsafe {
                // Set this as the rendered framebuffer, it then handles switching
                framebuffer.bind();

                // Use needs to be called before setting these even if you have the location
                // shader_program.use_program();
                // shader_program.set_float("material.shininess", 32.0);

                // Send light data to shader
                // dir_light.send_lighting_data(&shader_program);
                // point_light.send_lighting_data(&shader_program);
                // shader_program.set_float("pointLight.far_plane", 300.0); // Temp

                // Send some sample lights to lighting pass
                lighting_pass_shader_program.use_program();
                for (i, pos) in light_positions.iter().enumerate() {
                    lighting_pass_shader_program.set_vector_3(format!("lights[{}].Position", i).as_str(), pos, false).unwrap();
                    lighting_pass_shader_program.set_vector_3(format!("lights[{}].Color", i).as_str(), &light_colors[i], false).unwrap();
                    lighting_pass_shader_program.set_float(format!("lights[{}].Radius", i).as_str(), light_radii[i], false).unwrap();
                }

                // Already has a use program
                // TODO: simple rule should be to call use program before you pass it anywhere,
                // TODO: therefore to reduce calls to it, remove it from struct functions and
                // TODO: make it manual
                // TODO: Also make this a uniform buffer to reduce calls
                // dir_light.configure_shader_and_matrices(&depth_shader_program);
                // dir_light.configure_shader_and_matrices(&shader_program);

                // point_light.configure_shader_and_matrices(&cube_depth_shader_program);

                // Set projection for all shaders that require it
                uniform_buffer.write_data::<Matrix4<f32>>(projection_transform.as_ptr() as *const gl::types::GLvoid, 0);

                // Set exposure
                framebuffer_shader_program.use_program();
                framebuffer_shader_program.set_float("exposure", 0.2, false).unwrap();

                // Set light colour
                // TODO: this should be done based on what light is currently rendering instead
                // light_shader_program.use_program();
                // light_shader_program.set_vector_3("diffuse", &dir_light.diffuse);

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

            // // Fix peter panning
            // gl::CullFace(gl::FRONT);

            // // Draw to depth buffer for lighting
            // dir_light.bind_buffer();

            // depth_shader_program.use_program();
            // planet_model.draw(&depth_shader_program);
            // rock_model.draw(&depth_shader_program);
            // wall_quad.draw(&depth_shader_program);
            // toy_quad.draw(&depth_shader_program);

            // // Reset
            // gl::CullFace(gl::BACK);

            // // Floor, doesn't cast shadows so don't cull front faces
            // backpack_model.draw(&depth_shader_program);

            // // Fix peter panning
            // gl::CullFace(gl::FRONT);

            // // Draw to depth buffer for lighting
            // point_light.bind_buffer();

            // cube_depth_shader_program.use_program();
            // planet_model.draw(&cube_depth_shader_program);
            // rock_model.draw(&cube_depth_shader_program);
            // wall_quad.draw(&cube_depth_shader_program);
            // toy_quad.draw(&cube_depth_shader_program);

            // // Reset
            // gl::CullFace(gl::BACK);

            // // Floor, doesn't cast shadows so don't cull front faces
            // backpack_model.draw(&cube_depth_shader_program);

            // Draw to regular framebuffer for an actual scene
            framebuffer.bind();

            shader_program.use_program();
            shader_program.set_vector_3("viewPos", &camera.position.to_vec(), false);
            planet_model.draw(&shader_program).unwrap();

            if show_debug {
                debug_shader_program.use_program();
                planet_model.draw(&debug_shader_program).unwrap();
            }

            // END - DRAW MODELS HERE

            // Drawn last so it only is drawn over unused pixels, improving performance
            skybox.draw(&skybox_shader_program).unwrap();

            // Draw framebuffer
            framebuffer.draw(
                &framebuffer_shader_program,
                &blur_shader_program,
                &lighting_pass_shader_program
            );
        }

        window.swap_buffers();
        glfw.poll_events();
    }

    // TODO: Delete GL objects when they exit scope
}

// TODO: use this function for all logging so logging level can be changed easily
// TODO: e.g. min_severity property (throw options in a struct?)
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
    framebuffer: &mut RenderPipeline,
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
                    shader_programs[i].reload().unwrap();
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

                uniform_buffer.write_data::<Matrix4<f32>>(projection_transform.as_ptr() as *const gl::types::GLvoid, 0);
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