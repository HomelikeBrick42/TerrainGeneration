use crate::chunk::{Block, CHUNK_SIZE, Chunk, chunk_bind_group_layout};
use bytemuck::NoUninit;
use math::{Rotor, Transform, Vector3};
use rand::seq::IndexedRandom;
use std::{collections::HashSet, f32::consts::TAU};
use winit::keyboard::KeyCode;

#[derive(Debug, Clone, Copy, NoUninit)]
#[repr(C)]
struct GpuCamera {
    transform: Transform,
    near_plane: f32,
    aspect: f32,
}

pub struct Camera {
    position: Vector3<f32>,
    base_rotation: Rotor,
    xy_rotation: f32,
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

pub struct Game {
    camera: Camera,

    chunks: Vec<Chunk>,

    camera_buffer: wgpu::Buffer,
    camera_bind_group: wgpu::BindGroup,

    chunk_render_pipeline: wgpu::RenderPipeline,
}

impl Game {
    pub fn new(device: &wgpu::Device, queue: &wgpu::Queue) -> Self {
        let camera_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Camera Buffer"),
            size: size_of::<GpuCamera>().next_multiple_of(16) as _,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let camera_bind_group_layout =
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
            });
        let camera_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Camera Bind Group"),
            layout: &camera_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: camera_buffer.as_entire_binding(),
            }],
        });

        let chunk_shader = device.create_shader_module(wgpu::include_wgsl!(concat!(
            env!("OUT_DIR"),
            "/shaders/chunk.wgsl"
        )));
        let chunk_render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Chunk Render Pipeline Layout"),
                bind_group_layouts: &[&camera_bind_group_layout, &chunk_bind_group_layout(device)],
                push_constant_ranges: &[],
            });
        let chunk_render_pipeline =
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("Chunk Render Pipeline"),
                layout: Some(&chunk_render_pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &chunk_shader,
                    entry_point: Some("vertex"),
                    compilation_options: wgpu::PipelineCompilationOptions::default(),
                    buffers: &[],
                },
                primitive: wgpu::PrimitiveState {
                    topology: wgpu::PrimitiveTopology::TriangleStrip,
                    strip_index_format: None,
                    front_face: wgpu::FrontFace::Cw,
                    cull_mode: None,
                    unclipped_depth: false,
                    polygon_mode: wgpu::PolygonMode::Fill,
                    conservative: false,
                },
                depth_stencil: Some(wgpu::DepthStencilState {
                    format: wgpu::TextureFormat::Depth32Float,
                    depth_write_enabled: true,
                    depth_compare: wgpu::CompareFunction::Greater,
                    stencil: wgpu::StencilState::default(),
                    bias: wgpu::DepthBiasState::default(),
                }),
                multisample: wgpu::MultisampleState {
                    count: 1,
                    mask: !0,
                    alpha_to_coverage_enabled: false,
                },
                fragment: Some(wgpu::FragmentState {
                    module: &chunk_shader,
                    entry_point: Some("fragment"),
                    compilation_options: wgpu::PipelineCompilationOptions::default(),
                    targets: &[Some(wgpu::ColorTargetState {
                        format: wgpu::TextureFormat::Bgra8Unorm,
                        blend: None,
                        write_mask: wgpu::ColorWrites::all(),
                    })],
                }),
                multiview: None,
                cache: None,
            });

        let mut rng = rand::rng();

        let mut chunks = vec![];
        for chunk_z in 0..5 {
            for chunk_y in 0..5 {
                for chunk_x in 0..5 {
                    let mut chunk = Chunk::new(
                        device,
                        queue,
                        Vector3 {
                            x: (chunk_x * CHUNK_SIZE) as f32,
                            y: (chunk_y * CHUNK_SIZE) as f32,
                            z: (chunk_z * CHUNK_SIZE) as f32,
                        },
                    );
                    for block_z in 0..CHUNK_SIZE {
                        for block_y in 0..CHUNK_SIZE {
                            for block_x in 0..CHUNK_SIZE {
                                let x = ((chunk_x * CHUNK_SIZE) + block_x) as f32;
                                let y = ((chunk_y * CHUNK_SIZE) + block_y) as f32;
                                let z = ((chunk_z * CHUNK_SIZE) + block_z) as f32;

                                if y < ((x / 5.0).sin() * 0.5 + 0.5) * 20.0
                                    + ((z / 7.0).sin() * 0.5 + 0.5) * 10.0
                                    + 10.0
                                {
                                    *chunk.get_block_mut(block_x, block_y, block_z).unwrap() =
                                        *[Block::Red, Block::Green, Block::Blue]
                                            .choose(&mut rng)
                                            .unwrap();
                                }
                            }
                        }
                    }
                    chunks.push(chunk);
                }
            }
        }

        Self {
            camera: Camera::new(Vector3 {
                x: -2.0,
                y: 0.0,
                z: 0.0,
            }),

            chunks,

            camera_buffer,
            camera_bind_group,

            chunk_render_pipeline,
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

        for chunk in &mut self.chunks {
            chunk.pre_render(device, queue);
        }

        |render_pass| {
            for chunk in &self.chunks {
                chunk.render(
                    &self.chunk_render_pipeline,
                    &self.camera_bind_group,
                    self.camera.position,
                    render_pass,
                );
            }
        }
    }
}
