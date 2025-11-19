use bytemuck::NoUninit;
use math::{Rotor, Transform, Vector3};

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
