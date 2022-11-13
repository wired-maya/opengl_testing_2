use cgmath::{Matrix4, Vector3, Vector2};
use super::{Texture, Mesh, Vertex, Model};

pub fn create_quad(diff_map: Option<&str>, norm_map: Option<&str>, disp_map: Option<&str>, model_transforms: Vec<Matrix4<f32>>) -> Mesh {
    let mut textures: Vec<Texture> = vec![];
    
    if let Some(diff_path) = diff_map {
        textures.push(
            Texture {
                id: unsafe { Model::texture_from_file(diff_path) },
                type_: "diffuse".into(),
                path: diff_path.into()
            }
        );
    }
    if let Some(norm_path) = norm_map {
        textures.push(
            Texture {
                id: unsafe { Model::texture_from_file(norm_path) },
                type_: "normal".into(),
                path: norm_path.into()
            }
        )
    };
    if let Some(disp_path) = disp_map {
        textures.push(
            Texture {
                id: unsafe { Model::texture_from_file(disp_path) },
                type_: "displacement".into(),
                path: disp_path.into()
            }
        );
    }

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

    let mesh = Mesh::new(vertices, indices, textures, model_transforms);

    mesh
}