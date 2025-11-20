use std::io::Write;

use crate::{
    camera::{Camera, GpuCamera, camera_bind_group_layout},
    chunks::Chunks,
};
use math::Vector3;
use wgpu::naga::FastHashSet;
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

        Self {
            camera: Camera::new(Vector3 {
                x: 0.0,
                y: 10.0,
                z: 0.0,
            }),
            fake_camera_position: None,

            camera_buffer,
            camera_bind_group,

            chunks: Chunks::new(device, queue),
        }
    }

    pub fn update(&mut self, keys: &FastHashSet<KeyCode>, ts: f32) {
        print!("\rFPS: {:.3}                            ", 1.0 / ts);
        _ = std::io::stdout().flush();

        self.camera.update(keys, ts);

        if keys.contains(&KeyCode::KeyF) {
            self.fake_camera_position = Some(self.camera.position);
        }
        if keys.contains(&KeyCode::KeyG) {
            self.fake_camera_position = None;
        }

        self.chunks
            .load_unload_chunks(self.fake_camera_position.unwrap_or(self.camera.position));
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
