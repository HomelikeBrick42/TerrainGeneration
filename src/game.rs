use crate::gpu::StorageBuffer;
use bytemuck::NoUninit;

#[derive(Debug, Clone, Copy, NoUninit)]
#[repr(C)]
struct GpuCamera {
    aspect: f32,
}

#[derive(Debug, Clone, Copy, NoUninit)]
#[repr(C)]
struct Face {
    x: f32,
    y: f32,
    z: f32,
    red: f32,
    green: f32,
    blue: f32,
    width: f32,
    height: f32,
}

pub struct Game {
    camera_buffer: wgpu::Buffer,
    camera_bind_group: wgpu::BindGroup,

    faces_buffer: StorageBuffer<Face>,
    faces_bind_group_layout: wgpu::BindGroupLayout,
    faces_bind_group: wgpu::BindGroup,

    faces_render_pipeline: wgpu::RenderPipeline,

    faces: Vec<Face>,
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

        let faces_buffer = StorageBuffer::new(device, queue, "Faces Buffer", &[]);
        let faces_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Faces Bind Group Layout"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
            });
        let faces_bind_group =
            Self::faces_bind_group(device, &faces_bind_group_layout, faces_buffer.buffer());

        let faces_shader = device.create_shader_module(wgpu::include_wgsl!(concat!(
            env!("OUT_DIR"),
            "/shaders/faces.wgsl"
        )));

        let faces_render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Faces Render Pipeline Layout"),
                bind_group_layouts: &[&camera_bind_group_layout, &faces_bind_group_layout],
                push_constant_ranges: &[],
            });
        let faces_render_pipeline =
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("Faces Render Pipeline"),
                layout: Some(&faces_render_pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &faces_shader,
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
                depth_stencil: None,
                multisample: wgpu::MultisampleState {
                    count: 1,
                    mask: !0,
                    alpha_to_coverage_enabled: false,
                },
                fragment: Some(wgpu::FragmentState {
                    module: &faces_shader,
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

        Self {
            camera_buffer,
            camera_bind_group,

            faces_buffer,
            faces_bind_group_layout,
            faces_bind_group,

            faces_render_pipeline,

            faces: vec![Face {
                x: 0.0,
                y: 0.0,
                z: 0.0,
                red: 1.0,
                green: 1.0,
                blue: 1.0,
                width: 1.0,
                height: 1.0,
            }],
        }
    }

    pub fn update(&mut self, #[expect(unused)] ts: f32) {}

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
                aspect: width as f32 / height as f32,
            }),
        );

        if self.faces_buffer.write(device, queue, &self.faces) {
            self.faces_bind_group = Self::faces_bind_group(
                device,
                &self.faces_bind_group_layout,
                self.faces_buffer.buffer(),
            );
        }

        move |render_pass| {
            render_pass.set_pipeline(&self.faces_render_pipeline);
            render_pass.set_bind_group(0, &self.camera_bind_group, &[]);
            render_pass.set_bind_group(1, &self.faces_bind_group, &[]);
            render_pass.draw(0..4, 0..1);
        }
    }

    fn faces_bind_group(
        device: &wgpu::Device,
        layout: &wgpu::BindGroupLayout,
        faces_buffer: &wgpu::Buffer,
    ) -> wgpu::BindGroup {
        device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Faces Bind Group"),
            layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: faces_buffer.as_entire_binding(),
            }],
        })
    }
}
