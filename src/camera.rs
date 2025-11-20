use std::f32::consts::TAU;

use bytemuck::NoUninit;
use math::{Rotor, Transform, Vector3};
use wgpu::naga::FastHashSet;
use winit::keyboard::KeyCode;

#[derive(Debug, Clone, Copy, NoUninit)]
#[repr(C)]
pub struct GpuCamera {
    pub transform: Transform,
    pub near_plane: f32,
    pub aspect: f32,
}

pub struct Camera {
    pub position: Vector3<f32>,
    pub base_rotation: Rotor,
    pub xy_rotation: f32,
}

impl Camera {
    pub fn new(position: Vector3<f32>) -> Self {
        Self {
            position,
            base_rotation: Rotor::IDENTITY,
            xy_rotation: 0.0,
        }
    }

    pub fn transform(&self) -> Transform {
        Transform::from_rotor(Rotor::rotation_xy(self.xy_rotation).then(self.base_rotation))
            .then(Transform::translation(self.position))
    }

    pub fn update(&mut self, keys: &FastHashSet<KeyCode>, ts: f32) {
        let speed = 32.0;

        let forward = self.base_rotation.rotate_vector(Vector3 {
            x: 1.0,
            y: 0.0,
            z: 0.0,
        });
        let up = self.base_rotation.rotate_vector(Vector3 {
            x: 0.0,
            y: 1.0,
            z: 0.0,
        });
        let right = self.base_rotation.rotate_vector(Vector3 {
            x: 0.0,
            y: 0.0,
            z: 1.0,
        });

        if keys.contains(&KeyCode::KeyW) {
            self.position += forward * speed * ts;
        }
        if keys.contains(&KeyCode::KeyS) {
            self.position -= forward * speed * ts;
        }
        if keys.contains(&KeyCode::KeyA) {
            self.position -= right * speed * ts;
        }
        if keys.contains(&KeyCode::KeyD) {
            self.position += right * speed * ts;
        }
        if keys.contains(&KeyCode::KeyQ) {
            self.position -= up * speed * ts;
        }
        if keys.contains(&KeyCode::KeyE) {
            self.position += up * speed * ts;
        }

        let rotation_speed = TAU * 0.5;

        if keys.contains(&KeyCode::ArrowLeft) {
            self.base_rotation =
                Rotor::rotation_xz(-rotation_speed * ts).then(self.base_rotation);
        }
        if keys.contains(&KeyCode::ArrowRight) {
            self.base_rotation =
                Rotor::rotation_xz(rotation_speed * ts).then(self.base_rotation);
        }
        if keys.contains(&KeyCode::ArrowUp) {
            self.xy_rotation += rotation_speed * ts;
        }
        if keys.contains(&KeyCode::ArrowDown) {
            self.xy_rotation -= rotation_speed * ts;
        }
        self.xy_rotation = self.xy_rotation.clamp(TAU * -0.25, TAU * 0.25);
    }
}

pub(crate) fn camera_bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
    device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("Camera Bind Group Layout"),
        entries: &[wgpu::BindGroupLayoutEntry {
            binding: 0,
            visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        }],
    })
}
