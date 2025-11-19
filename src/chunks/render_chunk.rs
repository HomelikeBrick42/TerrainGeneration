use crate::{
    chunks::{CHUNK_SIZE, Chunks, Direction},
    gpu::StorageBuffer,
};
use bytemuck::NoUninit;
use enum_map::{EnumMap, enum_map};
use math::Vector3;

#[derive(Debug, Clone, Copy, NoUninit)]
#[repr(C)]
struct GpuChunk {
    x: f32,
    y: f32,
    z: f32,
}

#[derive(Debug, Clone, Copy, NoUninit)]
#[repr(C)]
struct GpuFace {
    x: f32,
    y: f32,
    z: f32,
    red: f32,
    green: f32,
    blue: f32,
    width: f32,
    height: f32,
}

struct Faces {
    count: u32,
    buffer: StorageBuffer<GpuFace>,
    bind_group_layout: wgpu::BindGroupLayout,
    bind_group: wgpu::BindGroup,
}

pub struct RenderChunk {
    chunk_buffer: wgpu::Buffer,
    faces: EnumMap<Direction, Faces>,
}

impl RenderChunk {
    pub fn new(device: &wgpu::Device, queue: &wgpu::Queue) -> Self {
        let chunk_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Chunk Buffer"),
            size: size_of::<GpuChunk>().next_multiple_of(16) as _,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let faces = enum_map! {
            _ => {
                let buffer = StorageBuffer::new(device, queue, "Faces Buffer", &[]);
                let bind_group_layout = chunk_bind_group_layout(device);
                let bind_group = chunk_bind_group(device, &bind_group_layout, &chunk_buffer, buffer.buffer());
                Faces {
                    count: 0,
                    buffer,
                    bind_group_layout,
                    bind_group,
                }
            },
        };

        Self {
            chunk_buffer,
            faces,
        }
    }

    pub fn rebuild(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        base_position: Vector3<i64>,
        chunks: &Chunks,
    ) {
        let mut face_buffers = enum_map! {
            _ => vec![],
        };

        for z in 0..CHUNK_SIZE as i64 {
            for y in 0..CHUNK_SIZE as i64 {
                for x in 0..CHUNK_SIZE as i64 {
                    let position = base_position + Vector3 { x, y, z };
                    let block = chunks.get_block(position).unwrap();
                    let (red, green, blue) = block.color();

                    for (direction, face_buffer) in &mut face_buffers {
                        if !block.soild_in_direction(direction) {
                            continue;
                        }

                        let (offset_x, offset_y, offset_z) = match direction {
                            Direction::PositiveX
                                if chunks
                                    .get_block(position + Vector3 { x: 1, y: 0, z: 0 })
                                    .is_none_or(|block| {
                                        !block.soild_in_direction(Direction::NegativeX)
                                    }) =>
                            {
                                (0.5, 0.0, 0.0)
                            }
                            Direction::NegativeX
                                if chunks
                                    .get_block(position - Vector3 { x: 1, y: 0, z: 0 })
                                    .is_none_or(|block| {
                                        !block.soild_in_direction(Direction::PositiveX)
                                    }) =>
                            {
                                (-0.5, 0.0, 0.0)
                            }

                            Direction::PositiveY
                                if chunks
                                    .get_block(position + Vector3 { x: 0, y: 1, z: 0 })
                                    .is_none_or(|block| {
                                        !block.soild_in_direction(Direction::NegativeY)
                                    }) =>
                            {
                                (0.0, 0.5, 0.0)
                            }
                            Direction::NegativeY
                                if chunks
                                    .get_block(position - Vector3 { x: 0, y: 1, z: 0 })
                                    .is_none_or(|block| {
                                        !block.soild_in_direction(Direction::PositiveY)
                                    }) =>
                            {
                                (0.0, -0.5, 0.0)
                            }

                            Direction::PositiveZ
                                if chunks
                                    .get_block(position + Vector3 { x: 0, y: 0, z: 1 })
                                    .is_none_or(|block| {
                                        !block.soild_in_direction(Direction::NegativeZ)
                                    }) =>
                            {
                                (0.0, 0.0, 0.5)
                            }
                            Direction::NegativeZ
                                if chunks
                                    .get_block(position - Vector3 { x: 0, y: 0, z: 1 })
                                    .is_none_or(|block| {
                                        !block.soild_in_direction(Direction::PositiveZ)
                                    }) =>
                            {
                                (0.0, 0.0, -0.5)
                            }

                            _ => continue,
                        };

                        face_buffer.push(GpuFace {
                            x: x as f32 + offset_x,
                            y: y as f32 + offset_y,
                            z: z as f32 + offset_z,
                            red,
                            green,
                            blue,
                            width: 1.0,
                            height: 1.0,
                        });
                    }
                }
            }
        }

        for (direction, faces) in &mut self.faces {
            let face_buffer = &face_buffers[direction];
            if faces.buffer.write(device, queue, face_buffer) {
                faces.bind_group = chunk_bind_group(
                    device,
                    &faces.bind_group_layout,
                    &self.chunk_buffer,
                    faces.buffer.buffer(),
                );
            }
            faces.count = face_buffer
                .len()
                .try_into()
                .expect("face count should be less than u32::MAX");
        }
    }

    pub fn render(
        &self,
        queue: &wgpu::Queue,
        camera_bind_group: &wgpu::BindGroup,
        chunk_render_pipeline: &wgpu::RenderPipeline,
        position: Vector3<f32>,
        camera_position: Vector3<f32>,
        render_pass: &mut wgpu::RenderPass<'_>,
    ) {
        queue.write_buffer(
            &self.chunk_buffer,
            0,
            bytemuck::bytes_of(&GpuChunk {
                x: position.x,
                y: position.y,
                z: position.z,
            }),
        );

        render_pass.set_pipeline(chunk_render_pipeline);
        render_pass.set_bind_group(0, camera_bind_group, &[]);
        for (direction, faces) in &self.faces {
            if faces.count == 0 {
                continue;
            }

            let direction = match direction {
                Direction::PositiveX => {
                    if camera_position.x > position.x - 1.0 {
                        0
                    } else {
                        continue;
                    }
                }
                Direction::NegativeX => {
                    if camera_position.x < position.x + CHUNK_SIZE as f32 - 1.0 {
                        1
                    } else {
                        continue;
                    }
                }
                Direction::PositiveY => {
                    if camera_position.y > position.y - 1.0 {
                        2
                    } else {
                        continue;
                    }
                }
                Direction::NegativeY => {
                    if camera_position.y < position.y + CHUNK_SIZE as f32 - 1.0 {
                        3
                    } else {
                        continue;
                    }
                }
                Direction::PositiveZ => {
                    if camera_position.z > position.z - 1.0 {
                        4
                    } else {
                        continue;
                    }
                }
                Direction::NegativeZ => {
                    if camera_position.z < position.z + CHUNK_SIZE as f32 - 1.0 {
                        5
                    } else {
                        continue;
                    }
                }
            };

            render_pass.set_bind_group(1, &faces.bind_group, &[]);
            render_pass.draw((direction << 3)..(direction << 3) | 4, 0..faces.count);
        }
    }
}

pub(crate) fn chunk_bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
    device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("Chunk Bind Group Layout"),
        entries: &[
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage { read_only: true },
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
        ],
    })
}

fn chunk_bind_group(
    device: &wgpu::Device,
    layout: &wgpu::BindGroupLayout,
    chunk_buffer: &wgpu::Buffer,
    faces_buffer: &wgpu::Buffer,
) -> wgpu::BindGroup {
    device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("Chunk Bind Group"),
        layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: chunk_buffer.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: faces_buffer.as_entire_binding(),
            },
        ],
    })
}
