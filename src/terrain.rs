use crate::chunks::Block;
use math::Vector3;
use rand::seq::IndexedRandom;

pub fn sin_wave(position: Vector3<i64>) -> Block {
    if (position.y as f32)
        < (position.x as f32 / 7.0).sin() * 10.0 + (position.z as f32 / 5.0).cos() * 10.0
    {
        *[Block::Red, Block::Green, Block::Blue]
            .choose(&mut rand::rng())
            .unwrap()
    } else {
        Block::Air
    }
}
