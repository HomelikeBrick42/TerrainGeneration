use crate::{
    camera::{Camera, GpuCamera, camera_bind_group_layout},
    chunks::{Block, CHUNK_SIZE, Chunk, Chunks},
};
use math::{Rotor, Vector3};
use rand::seq::IndexedRandom;
use std::{collections::HashSet, f32::consts::TAU};
use winit::keyboard::KeyCode;

pub struct Game {
    camera: Camera,
    fake_camera_position: Option<Vector3<f32>>,

    camera_buffer: wgpu::Buffer,
    camera_bind_group: wgpu::BindGroup,

    chunks: Chunks,
}

impl Game {
    pub fn new(device: &wgpu::Device, queue: &wgpu::Queue) -> Self {
        let camera_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Camera Buffer"),
            size: size_of::<GpuCamera>().next_multiple_of(16) as _,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let camera_bind_group_layout = camera_bind_group_layout(device);
        let camera_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Camera Bind Group"),
            layout: &camera_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: camera_buffer.as_entire_binding(),
            }],
        });

        let mut chunks = Chunks::new(device, queue);
        for x in -5..=5 {
            for y in -5..=5 {
                for z in -5..=5 {
                    let chunk_position = Vector3 { x, y, z };
                    chunks.insert_chunk(
                        chunk_position,
                        Chunk::with(|position| {
                            let x = chunk_position.x * CHUNK_SIZE as i64 + position.x as i64;
                            let y = chunk_position.y * CHUNK_SIZE as i64 + position.y as i64;
                            let z = chunk_position.z * CHUNK_SIZE as i64 + position.z as i64;

                            if (y as f32)
                                < (x as f32 / 7.0).sin() * 10.0 + (z as f32 / 5.0).cos() * 10.0
                            {
                                *[Block::Red, Block::Green, Block::Blue]
                                    .choose(&mut rand::rng())
                                    .unwrap()
                            } else {
                                Block::Air
                            }
                        }),
                    );
                }
            }
        }

        Self {
            camera: Camera::new(Vector3 {
                x: -2.0,
                y: 0.0,
                z: 0.0,
            }),
            fake_camera_position: None,

            camera_buffer,
            camera_bind_group,

            chunks,
        }
    }

    pub fn update(&mut self, keys: &HashSet<KeyCode>, ts: f32) {
        let speed = 32.0;

        let forward = self.camera.base_rotation.rotate_vector(Vector3 {
            x: 1.0,
            y: 0.0,
            z: 0.0,
        });
        let up = self.camera.base_rotation.rotate_vector(Vector3 {
            x: 0.0,
            y: 1.0,
            z: 0.0,
        });
        let right = self.camera.base_rotation.rotate_vector(Vector3 {
            x: 0.0,
            y: 0.0,
            z: 1.0,
        });

        if keys.contains(&KeyCode::KeyW) {
            self.camera.position += forward * speed * ts;
        }
        if keys.contains(&KeyCode::KeyS) {
            self.camera.position -= forward * speed * ts;
        }
        if keys.contains(&KeyCode::KeyA) {
            self.camera.position -= right * speed * ts;
        }
        if keys.contains(&KeyCode::KeyD) {
            self.camera.position += right * speed * ts;
        }
        if keys.contains(&KeyCode::KeyQ) {
            self.camera.position -= up * speed * ts;
        }
        if keys.contains(&KeyCode::KeyE) {
            self.camera.position += up * speed * ts;
        }

        let rotation_speed = TAU * 0.5;

        if keys.contains(&KeyCode::ArrowLeft) {
            self.camera.base_rotation =
                Rotor::rotation_xz(-rotation_speed * ts).then(self.camera.base_rotation);
        }
        if keys.contains(&KeyCode::ArrowRight) {
            self.camera.base_rotation =
                Rotor::rotation_xz(rotation_speed * ts).then(self.camera.base_rotation);
        }
        if keys.contains(&KeyCode::ArrowUp) {
            self.camera.xy_rotation += rotation_speed * ts;
        }
        if keys.contains(&KeyCode::ArrowDown) {
            self.camera.xy_rotation -= rotation_speed * ts;
        }
        self.camera.xy_rotation = self.camera.xy_rotation.clamp(TAU * -0.25, TAU * 0.25);

        if keys.contains(&KeyCode::KeyF) {
            self.fake_camera_position = Some(self.camera.position);
        }
        if keys.contains(&KeyCode::KeyG) {
            self.fake_camera_position = None;
        }
    }

    pub fn render<'a>(
        &'a mut self,
        device: &'a wgpu::Device,
        queue: &'a wgpu::Queue,
        #[expect(unused)] command_encoder: &mut wgpu::CommandEncoder,
        width: u32,
        height: u32,
        #[expect(unused)] dt: f32,
    ) -> impl FnOnce(&mut wgpu::RenderPass<'_>) + use<'a> {
        queue.write_buffer(
            &self.camera_buffer,
            0,
            bytemuck::bytes_of(&GpuCamera {
                transform: self.camera.transform(),
                near_plane: 0.1,
                aspect: width as f32 / height as f32,
            }),
        );

        self.chunks.pre_render(device, queue);

        |render_pass| {
            self.chunks.render(
                queue,
                &self.camera_bind_group,
                self.fake_camera_position.unwrap_or(self.camera.position),
                render_pass,
            );
        }
    }
}
