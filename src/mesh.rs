use cgmath::{Vector3, Vector2};

struct Vertex {
    position: Vector3<f32>,
    normal: Vector3<f32>,
    tex_coord: Vector2<f32>
}

struct Texture {
    id: u32,
    _type: String
}