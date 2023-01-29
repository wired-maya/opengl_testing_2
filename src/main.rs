extern crate gl;
extern crate glfw;
extern crate rand;
extern crate silver_gl;
extern crate cinema_skylight_engine;

use self::glfw::{Context, Key, Action};
use std::{sync::mpsc::Receiver, ffi::{c_void, CString}, slice, error::Error};
use silver_gl::*;
use cgmath::{vec4, vec2, Quaternion, Euler, Deg};
use cinema_skylight_engine::*;

const WIDTH: i32 = 1280;
const HEIGHT: i32 = 720;
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

    let framebuffer_shader_program = resource_manager.load_shader_program(
        ShaderPathBundle {
            vertex: Some("assets/shaders/framebuffer.vert".to_string()),
            geometry: None,
            fragment: Some("assets/shaders/framebuffer.frag".to_string())
        }
    )?;

    let mut scene = Widget2dScene::new(
        &mut resource_manager,
        ShaderPathBundle {
            vertex: Some("assets/shaders/widget.vert".to_string()),
            geometry: None,
            fragment: Some("assets/shaders/widget.frag".to_string())
        },
        CameraSize { width: WIDTH, height: HEIGHT, fov: FOV },
        Box::new(Widget2dRenderPipeline::new(WIDTH, HEIGHT)?),
        vec4(0.0, 1.0, 0.0, 1.0)
    )?;

    // Add a few widgets
    let square_1 = BackgroundWidget {
        colour: vec4(1.0, 0.0, 0.0, 1.0),
        position: vec2(0.3, 0.2),
        width: 0.2,
        height: 0.25,
        ..Default::default()
    };
    let square_2 = BackgroundWidget {
        colour: vec4(0.0, 0.0, 1.0, 1.0),
        position: vec2(0.15, 0.15),
        width: 0.25,
        height: 0.15,
        ..Default::default()
    };
    let square_3 = BackgroundWidget {
        colour: vec4(0.5, 0.5, 0.5, 1.0),
        position: vec2(0.5, 0.5),
        width: 0.2,
        height: 0.2,
        // rotation: Quaternion::new(0.92, 0.0, 0.0, -0.39),
        ..Default::default()
    };
    let square_4 = BackgroundWidget {
        colour: vec4(1.0, 0.0, 1.0, 1.0),
        position: vec2(0.25, 0.25),
        width: 0.5,
        height: 0.5,
        ..Default::default()
    };

    scene.top_widget.children.push(Box::new(square_1));
    scene.top_widget.children.push(Box::new(square_2));
    scene.top_widget.children.push(Box::new(square_3));
    scene.top_widget.children[2].get_children_mut().push(Box::new(square_4));


    scene.set_widget_tree()?;

    let mut default_framebuffer = Framebuffer::new_default(WIDTH, HEIGHT);
    default_framebuffer.link_to(scene.render_pipeline.get_link().unwrap());

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

        // Some fun transforms cuz why not
        scene.top_widget.children[0].set_position(vec2(0.3, 0.2 + (current_frame.sin() / 10.0)));
        scene.top_widget.children[1].set_size(0.25 + (current_frame.sin() / 5.0), 0.15 - (current_frame.sin() / 10.0));
        scene.top_widget.children[2].set_rotation(Quaternion::from(Euler::new(Deg(0.0), Deg(0.0), Deg(current_frame * 100.0))));

        scene.set_widget_transforms()?;

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
    scene: &mut Widget2dScene,
    default_framebuffer: &mut Framebuffer
) -> Result<(), EngineError> {
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

                // scene.camera.process_mouse_movement(x_offset, y_offset, true);
            }
            _ => {}
        }
    }

    // if window.get_key(Key::W) == Action::Press {
    //     scene.camera.process_movement(CameraMovement::FORWARD, delta_time);
    // }
    // if window.get_key(Key::S) == Action::Press {
    //     scene.camera.process_movement(CameraMovement::BACKWARD, delta_time);
    // }
    // if window.get_key(Key::A) == Action::Press {
    //     scene.camera.process_movement(CameraMovement::LEFT, delta_time);
    // }
    // if window.get_key(Key::D) == Action::Press {
    //     scene.camera.process_movement(CameraMovement::RIGHT, delta_time);
    // }
    // if window.get_key(Key::Space) == Action::Press {
    //     scene.camera.process_movement(CameraMovement::UP, delta_time);
    // }
    // if window.get_key(Key::LeftControl) == Action::Press {
    //     scene.camera.process_movement(CameraMovement::DOWN, delta_time);
    // }

    Ok(())
}