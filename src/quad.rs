use cgmath::{Matrix4, Vector3, Vector2};

use crate::{mesh::{Texture, Mesh, Vertex}, model::Model};

pub fn create_quad(diff_map: &str, norm_map: &str, disp_map: &str, model_transforms: Vec<Matrix4<f32>>) -> Mesh {
    let diff_tex = Texture {
        id: unsafe { Model::texture_from_file(diff_map) },
        type_: "diffuse".into(),
        path: diff_map.into()
    };
    let norm_tex = Texture {
        id: unsafe { Model::texture_from_file(norm_map) },
        type_: "normal".into(),
        path: norm_map.into()
    };
    let disp_tex = Texture {
        id: unsafe { Model::texture_from_file(disp_map) },
        type_: "displacement".into(),
        path: disp_map.into()
    };

    // Flat panel definition
    let vertices = vec![
        Vertex {
            position: Vector3::new(-1.0, 1.0, 0.0),
            normal: Vector3::new(0.0, 0.0, 1.0),
            tex_coord: Vector2::new(0.0, 1.0),
            ..Vertex::default()
        },
        Vertex {
            position: Vector3::new(-1.0, -1.0, 0.0),
            normal: Vector3::new(0.0, 0.0, 1.0),
            tex_coord: Vector2::new(0.0, 0.0),
            ..Vertex::default()
        },
        Vertex {
            position: Vector3::new(1.0, -1.0, 0.0),
            normal: Vector3::new(0.0, 0.0, 1.0),
            tex_coord: Vector2::new(1.0, 0.0),
            ..Vertex::default()
        },
        Vertex {
            position: Vector3::new(1.0, 1.0, 0.0),
            normal: Vector3::new(0.0, 0.0, 1.0),
            tex_coord: Vector2::new(1.0, 1.0),
            ..Vertex::default()
        }
    ];

    let indices = vec![
        0, 1, 2,
        0, 2, 3
    ];

    let textures = vec![
        diff_tex, norm_tex, disp_tex
    ];

    let mesh = Mesh::new(vertices, indices, textures, model_transforms);

    mesh
}