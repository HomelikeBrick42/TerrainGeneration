use crate::gpu::StorageBuffer;
use bytemuck::NoUninit;
use enum_map::{Enum, EnumMap, enum_map};

pub const CHUNK_SIZE: usize = 16;

#[derive(Debug, Clone, Copy, Enum)]
pub enum Direction {
    PositiveX,
    NegativeX,
    PositiveY,
    NegativeY,
    PositiveZ,
    NegativeZ,
}

#[derive(Debug, Clone, Copy)]
pub enum Block {
    Air,
    Solid,
}

impl Block {
    pub fn soild_in_direction(&self, #[expect(unused)] direction: Direction) -> bool {
        match *self {
            Block::Air => false,
            Block::Solid => true,
        }
    }

    pub fn color(&self) -> (f32, f32, f32) {
        match *self {
            Block::Air => (1.0, 1.0, 1.0),
            Block::Solid => (1.0, 1.0, 1.0),
        }
    }
}

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

pub struct Chunk {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    should_rebuild_chunks: bool,
    blocks: Box<[Block; CHUNK_SIZE * CHUNK_SIZE * CHUNK_SIZE]>,

    chunk_buffer: wgpu::Buffer,
    faces: EnumMap<Direction, Faces>,
}

impl Chunk {
    pub fn new(device: &wgpu::Device, queue: &wgpu::Queue, x: f32, y: f32, z: f32) -> Self {
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
            x,
            y,
            z,
            should_rebuild_chunks: true,
            blocks: std::iter::repeat_n(Block::Air, CHUNK_SIZE * CHUNK_SIZE * CHUNK_SIZE)
                .collect::<Vec<_>>()
                .try_into()
                .unwrap(),

            chunk_buffer,
            faces,
        }
    }

    pub fn get_block(&self, x: usize, y: usize, z: usize) -> Option<&Block> {
        if x < CHUNK_SIZE && y < CHUNK_SIZE && z < CHUNK_SIZE {
            Some(&self.blocks[x + y * CHUNK_SIZE + z * CHUNK_SIZE * CHUNK_SIZE])
        } else {
            None
        }
    }

    pub fn get_block_mut(&mut self, x: usize, y: usize, z: usize) -> Option<&mut Block> {
        if x < CHUNK_SIZE && y < CHUNK_SIZE && z < CHUNK_SIZE {
            self.should_rebuild_chunks = true;
            Some(&mut self.blocks[x + y * CHUNK_SIZE + z * CHUNK_SIZE * CHUNK_SIZE])
        } else {
            None
        }
    }

    pub fn pre_render(&mut self, device: &wgpu::Device, queue: &wgpu::Queue) {
        queue.write_buffer(
            &self.chunk_buffer,
            0,
            bytemuck::bytes_of(&GpuChunk {
                x: self.x,
                y: self.y,
                z: self.z,
            }),
        );

        if self.should_rebuild_chunks {
            self.should_rebuild_chunks = false;

            let mut face_buffers = enum_map! {
                _ => vec![],
            };

            for z in 0..CHUNK_SIZE {
                for y in 0..CHUNK_SIZE {
                    for x in 0..CHUNK_SIZE {
                        let block = self.get_block(x, y, z).unwrap();
                        let (red, green, blue) = block.color();

                        for (direction, face_buffer) in &mut face_buffers {
                            if !block.soild_in_direction(direction) {
                                continue;
                            }

                            let (offset_x, offset_y, offset_z) = match direction {
                                Direction::PositiveX
                                    if self.get_block(x.wrapping_add(1), y, z).is_none_or(
                                        |block| !block.soild_in_direction(Direction::NegativeX),
                                    ) =>
                                {
                                    (0.5, 0.0, 0.0)
                                }
                                Direction::NegativeX
                                    if self.get_block(x.wrapping_sub(1), y, z).is_none_or(
                                        |block| !block.soild_in_direction(Direction::PositiveX),
                                    ) =>
                                {
                                    (-0.5, 0.0, 0.0)
                                }

                                Direction::PositiveY
                                    if self.get_block(x, y.wrapping_add(1), z).is_none_or(
                                        |block| !block.soild_in_direction(Direction::NegativeY),
                                    ) =>
                                {
                                    (0.0, 0.5, 0.0)
                                }
                                Direction::NegativeY
                                    if self.get_block(x, y.wrapping_sub(1), z).is_none_or(
                                        |block| !block.soild_in_direction(Direction::PositiveY),
                                    ) =>
                                {
                                    (0.0, -0.5, 0.0)
                                }

                                Direction::PositiveZ
                                    if self.get_block(x, y, z.wrapping_add(1)).is_none_or(
                                        |block| !block.soild_in_direction(Direction::NegativeZ),
                                    ) =>
                                {
                                    (0.0, 0.0, 0.5)
                                }
                                Direction::NegativeZ
                                    if self.get_block(x, y, z.wrapping_sub(1)).is_none_or(
                                        |block| !block.soild_in_direction(Direction::PositiveZ),
                                    ) =>
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
    }

    pub fn render(
        &self,
        chunk_render_pipeline: &wgpu::RenderPipeline,
        camera_bind_group: &wgpu::BindGroup,
        camera_x: f32,
        camera_y: f32,
        camera_z: f32,
        render_pass: &mut wgpu::RenderPass<'_>,
    ) {
        render_pass.set_pipeline(chunk_render_pipeline);
        render_pass.set_bind_group(0, camera_bind_group, &[]);
        for (direction, faces) in &self.faces {
            let direction = match direction {
                Direction::PositiveX => {
                    if camera_x > self.x - 1.0 {
                        0
                    } else {
                        continue;
                    }
                }
                Direction::NegativeX => {
                    if camera_x < self.x + CHUNK_SIZE as f32 - 1.0 {
                        1
                    } else {
                        continue;
                    }
                }
                Direction::PositiveY => {
                    if camera_y > self.y - 1.0 {
                        2
                    } else {
                        continue;
                    }
                }
                Direction::NegativeY => {
                    if camera_y < self.y + CHUNK_SIZE as f32 - 1.0 {
                        3
                    } else {
                        continue;
                    }
                }
                Direction::PositiveZ => {
                    if camera_z > self.z - 1.0 {
                        4
                    } else {
                        continue;
                    }
                }
                Direction::NegativeZ => {
                    if camera_z < self.z + CHUNK_SIZE as f32 - 1.0 {
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
