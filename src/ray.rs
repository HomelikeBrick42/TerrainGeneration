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
    let step = Vector3 {
        x: ray.direction.x.signum() as i64,
        y: ray.direction.y.signum() as i64,
        z: ray.direction.z.signum() as i64,
    };
    let delta = Vector3 {
        x: (1.0 / ray.direction.x).abs(),
        y: (1.0 / ray.direction.y).abs(),
        z: (1.0 / ray.direction.z).abs(),
    };

    let mut pos = Vector3 {
        x: ray.origin.x.floor() as i64,
        y: ray.origin.y.floor() as i64,
        z: ray.origin.z.floor() as i64,
    };
    let mut t_max = Vector3 {
        x: (((pos.x + step.x.max(0)) as f32 - ray.origin.x) / ray.direction.x).abs(),
        y: (((pos.y + step.y.max(0)) as f32 - ray.origin.y) / ray.direction.y).abs(),
        z: (((pos.z + step.z.max(0)) as f32 - ray.origin.z) / ray.direction.z).abs(),
    };

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
