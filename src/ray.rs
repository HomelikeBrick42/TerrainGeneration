use crate::chunks::Chunks;
use math::Vector3;

#[derive(Debug, Clone, Copy)]
pub struct Ray {
    pub origin: Vector3<f32>,
    pub direction: Vector3<f32>,
}

#[derive(Debug, Clone, Copy)]
pub struct Hit {
    pub block: Vector3<i64>,
    pub position: Vector3<f32>,
    pub distance: f32,
}

pub fn raycast(chunks: &Chunks, ray: Ray, max_distance: f32) -> Option<Hit> {
    let step = ray.direction.map(|e| e.signum() as i64);
    let delta = ray.direction.map(|e| (1.0 / e).abs());

    let mut pos = ray.origin.map(|e| e.floor() as i64);
    let mut t_max =
        (((pos + step).map(|e| e as f32) - ray.origin) / ray.direction).map(|e| e.abs());

    loop {
        let distance = t_max.x.min(t_max.y).min(t_max.z);
        if distance > max_distance {
            break;
        }

        let Some(block) = chunks.get_block(pos) else {
            break;
        };
        if block.raycast_solid() {
            return Some(Hit {
                block: pos,
                position: ray.origin + ray.direction * distance,
                distance,
            });
        }

        #[expect(clippy::collapsible_else_if)]
        if t_max.x < t_max.y {
            if t_max.x < t_max.z {
                pos.x += step.x;
                t_max.x += delta.x;
            } else {
                pos.z += step.z;
                t_max.z += delta.z;
            }
        } else {
            if t_max.y < t_max.z {
                pos.y += step.y;
                t_max.y += delta.y;
            } else {
                pos.z += step.z;
                t_max.z += delta.z;
            }
        }
    }

    None
}
