extern crate gl;
extern crate glfw;
extern crate rand;
extern crate silver_gl;
extern crate cinema_skylight_engine;

use self::glfw::{Context, Key, Action};
use std::{sync::mpsc::Receiver, ffi::{c_void, CString}, slice, error::Error};
use silver_gl::*;
use cgmath::{vec3, Point3, Matrix4, Vector3};
use self::rand::Rng;
use cinema_skylight_engine::*;

const WIDTH: i32 = 800;
const HEIGHT: i32 = 600;
const FOV: f32 = 45.0;

fn main() -> Result<(), Box<dyn Error>> {
    // Timing
    let mut delta_time: f32; // Time between current frame and last frame
    let mut last_frame: f32 = 0.0;

    let mut last_x = WIDTH as f32 / 2.0;
    let mut last_y = HEIGHT as f32 / 2.0;
    let mut first_mouse = true;

    let window_conf = WindowConfig {
        width: WIDTH as u32,
        height: HEIGHT as u32,
        title: String::from("engine test")
    };

    let mut gl_window = EngineWindow::new(window_conf);

    // Set all OpenGL parameters
    unsafe {
        // Create GL context
        gl::load_with(|symbol| gl_window.window.get_proc_address(symbol) as *const _);

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
    }

    let mut resource_manager = ResourceManager::default();

    let model_shader_program = resource_manager.load_shader_program(
        ShaderPathBundle {
            vertex: Some("assets/shaders/shader.vert".to_string()),
            geometry: None,
            fragment: Some("assets/shaders/deffered_shader.frag".to_string())
        }
    )?;
    let framebuffer_shader_program = resource_manager.load_shader_program(
        ShaderPathBundle {
            vertex: Some("assets/shaders/framebuffer.vert".to_string()),
            geometry: None,
            fragment: Some("assets/shaders/framebuffer.frag".to_string())
        }
    )?;
    let skybox_shader_program = resource_manager.load_shader_program(
        ShaderPathBundle {
            vertex: Some("assets/shaders/skybox.vert".to_string()),
            geometry: None,
            fragment: Some("assets/shaders/skybox.frag".to_string())
        }
    )?;
    let blur_shader_program = resource_manager.load_shader_program(
        ShaderPathBundle {
            vertex: Some("assets/shaders/framebuffer.vert".to_string()),
            geometry: None,
            fragment: Some("assets/shaders/gaussian_blur.frag".to_string())
        }
    )?;
    let lighting_pass_shader_program = resource_manager.load_shader_program(
        ShaderPathBundle {
            vertex: Some("assets/shaders/framebuffer.vert".to_string()),
            geometry: None,
            fragment: Some("assets/shaders/lighting_pass_shader.frag".to_string())
        }
    )?;

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

    let planet_model = resource_manager.load_model("assets/models/planet/planet.obj")?;
    // Temp until gameobject system is created
    planet_model.borrow_mut().tbo.set_data_mut(planet_transforms);

    let skybox = resource_manager.load_skybox("assets/textures/skybox/full.jpg")?;

    let render_pipeline = View3DRenderPipeline::new(
        WIDTH,
        HEIGHT,
        lighting_pass_shader_program,
        blur_shader_program
    )?;

    let mut default_framebuffer = Framebuffer::new_default(WIDTH, HEIGHT);

    default_framebuffer.link_to(render_pipeline.get_link().unwrap());

    let mut camera = Camera::new(WIDTH, HEIGHT, FOV, vec![&model_shader_program, &skybox_shader_program]).unwrap();
    camera.position = Point3 { x: 0.0, y: 0.0, z: 1.0 };

    let mut scene = View3DScene {
        models: vec![planet_model],
        model_shader_program,
        skybox,
        skybox_shader_program,
        camera,
        render_pipeline: Box::new(render_pipeline),
    };

    // Set exposure
    framebuffer_shader_program.use_program();
    framebuffer_shader_program.set_float("exposure", 0.2).unwrap();

    // Render loop, each iteration is a frame
    while !gl_window.window.should_close() {
        let current_frame = gl_window.glfw.get_time() as f32;
        delta_time = current_frame - last_frame;
        last_frame = current_frame;

        process_events(
            &mut gl_window.window,
            &gl_window.events,
            delta_time,
            &mut last_x,
            &mut last_y,
            &mut first_mouse,
            &mut scene,
            &mut default_framebuffer
        )?;

        let mut matrix = Matrix4::<f32>::from_translation(vec3(0.0, current_frame.sin(), 0.0));
        matrix = matrix * Matrix4::<f32>::from_scale(distance_scale / 10.0);
        scene.models[0].borrow_mut().tbo.set_data_index(matrix, 0);

        scene.draw()?;
        default_framebuffer.draw(&framebuffer_shader_program)?;

        // You can get a window pointer, you might be able to use that to have multithreading
        gl_window.window.swap_buffers(); // Can be called from separate threads apparently?
        gl_window.glfw.poll_events();
    }

    Ok(())
}

// TODO: Global logging level enum that is checked here and other places to see how much to log
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
    delta_time: f32,
    last_x: &mut f32,
    last_y: &mut f32,
    first_mouse: &mut bool,
    scene: &mut View3DScene,
    default_framebuffer: &mut Framebuffer
) -> Result<(), GlError> {
    for (_, event) in glfw::flush_messages(events) {
        match event {
            glfw::WindowEvent::FramebufferSize(width, height) => {
                scene.set_size(width, height)?;
                default_framebuffer.set_size(width, height)?;
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

                scene.camera.process_mouse_movement(x_offset, y_offset, true);
            }
            _ => {}
        }
    }

    if window.get_key(Key::W) == Action::Press {
        scene.camera.process_movement(CameraMovement::FORWARD, delta_time);
    }
    if window.get_key(Key::S) == Action::Press {
        scene.camera.process_movement(CameraMovement::BACKWARD, delta_time);
    }
    if window.get_key(Key::A) == Action::Press {
        scene.camera.process_movement(CameraMovement::LEFT, delta_time);
    }
    if window.get_key(Key::D) == Action::Press {
        scene.camera.process_movement(CameraMovement::RIGHT, delta_time);
    }
    if window.get_key(Key::Space) == Action::Press {
        scene.camera.process_movement(CameraMovement::UP, delta_time);
    }
    if window.get_key(Key::LeftControl) == Action::Press {
        scene.camera.process_movement(CameraMovement::DOWN, delta_time);
    }

    Ok(())
}