use crate::chunks::Block;
use math::Vector3;
use noise::{NoiseFn, Simplex};

pub fn hills(position: Vector3<i64>) -> Block {
    let position = Vector3 {
        x: position.x as f64,
        y: position.y as f64,
        z: position.z as f64,
    };

    let simplex = Simplex::new(1);

    let caves_scale = 50.0;
    let caves = simplex.get([
        position.x / caves_scale,
        position.y / caves_scale,
        position.z / caves_scale,
    ]);

    let hills_scale = 50.0;
    let hills_height = 10.0;
    let height = simplex.get([position.x / hills_scale, position.z / hills_scale]) * hills_height;

    if position.y < height && caves < 0.3 {
        if position.y < height - 5.0 {
            Block::Stone
        } else if position.y < height - 1.0 {
            Block::Dirt
        } else {
            Block::Grass
        }
    } else {
        Block::Air
    }
}
