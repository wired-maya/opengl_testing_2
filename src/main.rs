extern crate glfw;
extern crate rand;
extern crate silver_gl;
extern crate cinema_skylight_engine;

use glfw::{Context, Key, Action};
use std::{sync::mpsc::Receiver, ffi::{c_void, CString}, slice, error::Error, rc::Rc};
use silver_gl::*;
use cgmath::{vec4, vec2, Quaternion, Euler, Deg, vec3};
use cinema_skylight_engine::*;
use cinema_skylight_engine::widgets::primitives::*;

const WIDTH: i32 = 1920;
const HEIGHT: i32 = 1080;
const FOV: f32 = 45.0;

fn main() -> Result<(), Box<dyn Error>> {
    // Timing
    let mut delta_time: f32; // Time between current frame and last frame
    let mut last_frame: f32 = 0.0;

    let mut last_x = WIDTH as f32 / 2.0;
    let mut last_y = HEIGHT as f32 / 2.0;
    let mut first_mouse = true;

    let engine_conf = CSEngineConfig {
        width: WIDTH,
        height: HEIGHT,
        fov: FOV,
        title: String::from("engine test"),
        capture_mouse: false,
        ..Default::default()
    };

    let mut engine = CSEngine::new(engine_conf);

    let framebuffer_shader_program = engine.resource_manager.load_shader_program(
        ShaderPathBundle {
            vertex: Some("assets/shaders/framebuffer.vert".to_string()),
            geometry: None,
            fragment: Some("assets/shaders/framebuffer.frag".to_string())
        }
    )?;

    let background_shader_program = engine.resource_manager.load_shader_program(
        ShaderPathBundle {
            vertex: Some("assets/shaders/widgets/widget.vert".to_string()),
            geometry: None,
            fragment: Some("assets/shaders/widgets/background.frag".to_string())
        }
    )?;

    let border_shader_program = engine.resource_manager.load_shader_program(
        ShaderPathBundle {
            vertex: Some("assets/shaders/widgets/widget.vert".to_string()),
            geometry: None,
            fragment: Some("assets/shaders/widgets/border.frag".to_string())
        }
    )?;

    let texture_shader_program = engine.resource_manager.load_shader_program(
        ShaderPathBundle {
            vertex: Some("assets/shaders/widgets/widget.vert".to_string()),
            geometry: None,
            fragment: Some("assets/shaders/widgets/texture.frag".to_string())
        }
    )?;

    let pass_shader_program = engine.resource_manager.load_shader_program(
        ShaderPathBundle {
            vertex: Some("assets/shaders/widgets/widget.vert".to_string()),
            geometry: None,
            fragment: Some("assets/shaders/widgets/pass.frag".to_string())
        }
    )?;

    // let render_pipeline = View3DRenderPipeline::new(
    //     &mut resource_manager,
    //     WIDTH,
    //     HEIGHT,
    //     ShaderPathBundle {
    //         vertex: Some("assets/shaders/framebuffer.vert".to_string()),
    //         geometry: None,
    //         fragment: Some("assets/shaders/lighting_pass_shader.frag".to_string())
    //     },
    //     ShaderPathBundle {
    //         vertex: Some("assets/shaders/framebuffer.vert".to_string()),
    //         geometry: None,
    //         fragment: Some("assets/shaders/gaussian_blur.frag".to_string())
    //     }
    // )?;

    // let mut scene_3d = View3DScene::new(
    //     &mut resource_manager,
    //     ShaderPathBundle {
    //         vertex: Some("assets/shaders/shader.vert".to_string()),
    //         geometry: None,
    //         fragment: Some("assets/shaders/deffered_shader.frag".to_string())
    //     },
    //     "assets/textures/skybox/full.jpg",
    //     ShaderPathBundle {
    //         vertex: Some("assets/shaders/skybox.vert".to_string()),
    //         geometry: None,
    //         fragment: Some("assets/shaders/skybox.frag".to_string())
    //     },
    //     CameraSize { width: WIDTH, height: HEIGHT, fov: FOV },
    //     Box::new(render_pipeline)
    // )?;

    // let distance_scale = 2.0;
    // let amount = 4;

    // let planet_model = resource_manager.load_model("assets/models/planet/planet.obj")?;

    // for x in 0..amount {
    //     for z in 0..amount {
    //         let mut planet_obj = GameObject::from_model(Rc::clone(&planet_model));

    //         planet_obj.position = vec3(x as f32, 0.0, z as f32) * distance_scale;
    //         planet_obj.scale = distance_scale / 10.0;

    //         scene_3d.world_obj.children.push(planet_obj);
    //     }
    // }

    // scene_3d.models.push(planet_model);

    let background = BackgroundWidget::new(Rc::clone(&background_shader_program), vec4(0.0, 1.0, 0.0, 1.0));

    let mut scene = Widget2dScene::new(WIDTH, HEIGHT)?;

    let mut picture = PictureWidget::new(
        pass_shader_program,
        background_shader_program,
        texture_shader_program,
        border_shader_program,
        vec4(0.1, 0.1, 0.1, 0.1),
        vec4(1.0, 0.0, 0.0, 1.0),
        engine.resource_manager.load_texture_2d("assets/textures/grass.png")?,
        vec4(0.0, 0.0, 1.0, 1.0),
        vec4(0.1, 0.1, 0.1, 0.1)
    );

    picture.set_size(0.5, 0.5);
    picture.set_position(vec2(0.25, 0.25));

    scene.children.push(Box::new(background));
    scene.children.push(Box::new(picture));

    let mut default_framebuffer = Framebuffer::new_default(WIDTH, HEIGHT);
    default_framebuffer.link_to(scene.render_pipeline.get_link().unwrap());
    // default_framebuffer.link_to(scene_3d.render_pipeline.get_link().unwrap());

    // Set exposure
    framebuffer_shader_program.use_program();
    framebuffer_shader_program.set_float("exposure", 0.2).unwrap();

    // Render loop, each iteration is a frame
    while !engine.window.should_close() {
        let current_frame = engine.glfw.get_time() as f32;
        delta_time = current_frame - last_frame;
        last_frame = current_frame;

        process_events(
            &mut engine.window,
            &engine.events,
            delta_time,
            &mut last_x,
            &mut last_y,
            &mut first_mouse,
            &mut scene,
            // &mut scene_3d,
            &mut default_framebuffer
        )?;

        // First planet
        // scene_3d.world_obj.children[0].position = vec3(0.0, current_frame.sin(), 0.0);
        // scene_3d.world_obj.children[0].scale = (current_frame * 3.0).cos() * 0.3;
        // scene_3d.world_obj.children[0].rotation = Quaternion::from(Euler::new(Deg(0.0), Deg(current_frame * 100.0), Deg(0.0)));

        // scene_3d.draw()?;

        // Some fun transforms cuz why not
        // scene.widget.get_children_mut()[0].set_position(vec2(0.3, 0.2 + (current_frame.sin() / 10.0)));
        scene.children[1].set_size(0.25 + (current_frame.sin() / 5.0), 0.15 - (current_frame.sin() / 10.0));
        // scene.widget.get_children_mut()[2].set_rotation(Quaternion::from(Euler::new(Deg(0.0), Deg(0.0), Deg(current_frame * 100.0))));

        scene.draw()?;

        framebuffer_shader_program.use_program();
        default_framebuffer.draw(&framebuffer_shader_program)?;

        // You can get a window pointer, you might be able to use that to have multithreading
        // https://docs.rs/glfw/0.51.0/glfw/struct.RenderContext.html
        engine.window.swap_buffers(); // Can be called from separate threads apparently?
        engine.glfw.poll_events();
    }

    Ok(())
}

fn process_events(
    window: &mut glfw::Window,
    events: &Receiver<(f64, glfw::WindowEvent)>,
    delta_time: f32,
    last_x: &mut f32,
    last_y: &mut f32,
    first_mouse: &mut bool,
    scene: &mut Widget2dScene,
    // scene_3d: &mut View3DScene,
    default_framebuffer: &mut Framebuffer
) -> Result<(), EngineError> {
    for (_, event) in glfw::flush_messages(events) {
        match event {
            glfw::WindowEvent::FramebufferSize(width, height) => {
                // scene_3d.set_size(width, height)?;
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

                // scene_3d.camera.process_mouse_movement(x_offset, y_offset, true);
            }
            _ => {}
        }
    }

    // if window.get_key(Key::W) == Action::Press {
    //     scene_3d.camera.process_movement(CameraMovement::FORWARD, delta_time);
    // }
    // if window.get_key(Key::S) == Action::Press {
    //     scene_3d.camera.process_movement(CameraMovement::BACKWARD, delta_time);
    // }
    // if window.get_key(Key::A) == Action::Press {
    //     scene_3d.camera.process_movement(CameraMovement::LEFT, delta_time);
    // }
    // if window.get_key(Key::D) == Action::Press {
    //     scene_3d.camera.process_movement(CameraMovement::RIGHT, delta_time);
    // }
    // if window.get_key(Key::Space) == Action::Press {
    //     scene_3d.camera.process_movement(CameraMovement::UP, delta_time);
    // }
    // if window.get_key(Key::LeftControl) == Action::Press {
    //     scene_3d.camera.process_movement(CameraMovement::DOWN, delta_time);
    // }

    Ok(())
}