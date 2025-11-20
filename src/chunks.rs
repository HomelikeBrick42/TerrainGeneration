use crate::{
    camera::camera_bind_group_layout,
    chunks::render_chunk::{RenderChunk, chunk_bind_group_layout},
    terrain,
};
use enum_map::Enum;
use math::Vector3;
use std::sync::mpsc::{Receiver, Sender};
use wgpu::naga::{FastHashMap, FastHashSet};

mod render_chunk;

pub const CHUNK_SIZE: u64 = 16;

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
    Red,
    Green,
    Blue,
}

impl Block {
    pub fn soild_in_direction(&self, #[expect(unused)] direction: Direction) -> bool {
        match *self {
            Block::Air => false,
            Block::Red | Block::Green | Block::Blue => true,
        }
    }

    pub fn color(&self) -> (f32, f32, f32) {
        match *self {
            Block::Air => (1.0, 1.0, 1.0),
            Block::Red => (1.0, 0.0, 0.0),
            Block::Green => (0.0, 1.0, 0.0),
            Block::Blue => (0.0, 0.0, 1.0),
        }
    }
}

pub struct Chunk {
    blocks: Box<[Block; (CHUNK_SIZE * CHUNK_SIZE * CHUNK_SIZE) as usize]>,
}

impl Chunk {
    pub fn with(mut f: impl FnMut(Vector3<u64>) -> Block) -> Self {
        Self {
            blocks: std::iter::repeat_n((), (CHUNK_SIZE * CHUNK_SIZE * CHUNK_SIZE) as usize)
                .enumerate()
                .map(|(index, ())| {
                    let x = index % CHUNK_SIZE as usize;
                    let y = (index / CHUNK_SIZE as usize) % CHUNK_SIZE as usize;
                    let z = index / (CHUNK_SIZE as usize * CHUNK_SIZE as usize);
                    f(Vector3 {
                        x: x as u64,
                        y: y as u64,
                        z: z as u64,
                    })
                })
                .collect::<Vec<_>>()
                .try_into()
                .unwrap(),
        }
    }

    pub fn filled(block: Block) -> Self {
        Self::with(|_| block)
    }

    pub fn get_block(&self, position: Vector3<u64>) -> Option<&Block> {
        if position.x < CHUNK_SIZE && position.y < CHUNK_SIZE && position.z < CHUNK_SIZE {
            Some(
                &self.blocks[(position.x
                    + position.y * CHUNK_SIZE
                    + position.z * (CHUNK_SIZE * CHUNK_SIZE))
                    as usize],
            )
        } else {
            None
        }
    }

    pub fn get_block_mut(&mut self, position: Vector3<u64>) -> Option<&mut Block> {
        if position.x < CHUNK_SIZE && position.y < CHUNK_SIZE && position.z < CHUNK_SIZE {
            Some(
                &mut self.blocks[(position.x
                    + position.y * CHUNK_SIZE
                    + position.z * (CHUNK_SIZE * CHUNK_SIZE))
                    as usize],
            )
        } else {
            None
        }
    }
}

pub struct Chunks {
    chunks: FastHashMap<Vector3<i64>, Chunk>,
    render_chunks: FastHashMap<Vector3<i64>, RenderChunk>,
    changed_chunks: FastHashSet<Vector3<i64>>,

    scheduled_chunk_loading: FastHashSet<Vector3<i64>>,
    loaded_chunk_sender: Sender<(Vector3<i64>, Chunk)>,
    loaded_chunk_receiver: Receiver<(Vector3<i64>, Chunk)>,

    chunk_render_pipeline: wgpu::RenderPipeline,
}

impl Chunks {
    pub fn new(device: &wgpu::Device, #[expect(unused)] queue: &wgpu::Queue) -> Self {
        let (loaded_chunk_sender, loaded_chunk_receiver) = std::sync::mpsc::channel();

        Self {
            chunks: FastHashMap::default(),
            render_chunks: FastHashMap::default(),
            changed_chunks: FastHashSet::default(),

            scheduled_chunk_loading: FastHashSet::default(),
            loaded_chunk_sender,
            loaded_chunk_receiver,

            chunk_render_pipeline: {
                let chunk_shader = device.create_shader_module(wgpu::include_wgsl!(concat!(
                    env!("OUT_DIR"),
                    "/shaders/chunk.wgsl"
                )));
                let chunk_render_pipeline_layout =
                    device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                        label: Some("Chunk Render Pipeline Layout"),
                        bind_group_layouts: &[
                            &camera_bind_group_layout(device),
                            &chunk_bind_group_layout(device),
                        ],
                        push_constant_ranges: &[],
                    });
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
                        cull_mode: Some(wgpu::Face::Back),
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
                })
            },
        }
    }

    pub fn load_unload_chunks(&mut self, camera_position: Vector3<f32>) {
        let center_chunk_position = Vector3 {
            x: (camera_position.x as i64).div_euclid(CHUNK_SIZE as i64),
            y: (camera_position.y as i64).div_euclid(CHUNK_SIZE as i64),
            z: (camera_position.z as i64).div_euclid(CHUNK_SIZE as i64),
        };

        let chunk_distance = 10;
        let generated_chunks_limit = 20usize;
        let deleted_chunks_limit = 20usize;

        {
            let mut to_delete = vec![];
            for &chunk_position in self.chunks.keys() {
                let relative_chunk_position = chunk_position - center_chunk_position;
                if relative_chunk_position.x.abs() > chunk_distance
                    || relative_chunk_position.y.abs() > chunk_distance
                    || relative_chunk_position.z.abs() > chunk_distance
                {
                    self.scheduled_chunk_loading.remove(&chunk_position);
                    to_delete.push((chunk_position, relative_chunk_position));
                }
            }
            to_delete.sort_unstable_by_key(|&(_, relative_position)| {
                std::cmp::Reverse(
                    relative_position.x.abs()
                        + relative_position.y.abs()
                        + relative_position.z.abs(),
                )
            });
            for (chunk_position, _) in to_delete.into_iter().take(deleted_chunks_limit) {
                self.chunks.remove(&chunk_position);
                self.render_chunks.remove(&chunk_position);
            }
        }

        {
            let mut to_load = vec![];
            for x in
                center_chunk_position.x - chunk_distance..=center_chunk_position.x + chunk_distance
            {
                for y in center_chunk_position.y - chunk_distance
                    ..=center_chunk_position.y + chunk_distance
                {
                    for z in center_chunk_position.z - chunk_distance
                        ..=center_chunk_position.z + chunk_distance
                    {
                        let chunk_position = Vector3 { x, y, z };
                        if self.chunks.contains_key(&chunk_position)
                            || self.scheduled_chunk_loading.contains(&chunk_position)
                        {
                            continue;
                        }

                        to_load.push((chunk_position, chunk_position - center_chunk_position));
                    }
                }
            }
            to_load.sort_unstable_by_key(|&(_, relative_position)| {
                relative_position.x.abs() + relative_position.y.abs() + relative_position.z.abs()
            });
            for (chunk_position, _) in to_load.into_iter() {
                if self.scheduled_chunk_loading.len() >= generated_chunks_limit {
                    break;
                }

                self.scheduled_chunk_loading.insert(chunk_position);
                let sender = self.loaded_chunk_sender.clone();
                rayon::spawn(move || {
                    let chunk = Chunk::with(|block_position| {
                        let position = Vector3 {
                            x: chunk_position.x * CHUNK_SIZE as i64 + block_position.x as i64,
                            y: chunk_position.y * CHUNK_SIZE as i64 + block_position.y as i64,
                            z: chunk_position.z * CHUNK_SIZE as i64 + block_position.z as i64,
                        };
                        terrain::sin_wave(position)
                    });
                    _ = sender.send((chunk_position, chunk));
                });
            }
        }

        for (chunk_position, chunk) in self.loaded_chunk_receiver.try_iter() {
            if !self.scheduled_chunk_loading.remove(&chunk_position)
                || self.chunks.contains_key(&chunk_position)
            {
                continue;
            }

            self.chunks.insert(chunk_position, chunk);
            self.changed_chunks.insert(chunk_position);
        }
    }

    pub fn get_block(&self, position: Vector3<i64>) -> Option<&Block> {
        let chunk_position = Vector3 {
            x: position.x.div_euclid(CHUNK_SIZE as i64),
            y: position.y.div_euclid(CHUNK_SIZE as i64),
            z: position.z.div_euclid(CHUNK_SIZE as i64),
        };
        let block_position = Vector3 {
            x: position.x.rem_euclid(CHUNK_SIZE as i64) as u64,
            y: position.y.rem_euclid(CHUNK_SIZE as i64) as u64,
            z: position.z.rem_euclid(CHUNK_SIZE as i64) as u64,
        };
        let block = self
            .chunks
            .get(&chunk_position)?
            .get_block(block_position)?;
        Some(block)
    }

    pub fn get_block_mut(&mut self, position: Vector3<i64>) -> Option<&mut Block> {
        let chunk_position = Vector3 {
            x: position.x.div_euclid(CHUNK_SIZE as i64),
            y: position.y.div_euclid(CHUNK_SIZE as i64),
            z: position.z.div_euclid(CHUNK_SIZE as i64),
        };
        let block_position = Vector3 {
            x: position.x.rem_euclid(CHUNK_SIZE as i64) as u64,
            y: position.y.rem_euclid(CHUNK_SIZE as i64) as u64,
            z: position.z.rem_euclid(CHUNK_SIZE as i64) as u64,
        };
        let block = self
            .chunks
            .get_mut(&chunk_position)?
            .get_block_mut(block_position)?;
        self.changed_chunks.insert(chunk_position);
        Some(block)
    }

    pub fn pre_render(&mut self, device: &wgpu::Device, queue: &wgpu::Queue) {
        let mut render_chunks = core::mem::take(&mut self.render_chunks);
        for position in core::mem::take(&mut self.changed_chunks)
            .into_iter()
            .flat_map(|position| {
                [
                    position,
                    position + Vector3 { x: 1, y: 0, z: 0 },
                    position - Vector3 { x: 1, y: 0, z: 0 },
                    position + Vector3 { x: 0, y: 1, z: 0 },
                    position - Vector3 { x: 0, y: 1, z: 0 },
                    position + Vector3 { x: 0, y: 0, z: 1 },
                    position - Vector3 { x: 0, y: 0, z: 1 },
                ]
            })
            .collect::<FastHashSet<_>>()
        {
            if !self.chunks.contains_key(&position) {
                render_chunks.remove(&position);
                continue;
            }
            let render_chunk = render_chunks
                .entry(position)
                .or_insert_with(|| RenderChunk::new(device, queue));
            render_chunk.rebuild(device, queue, position * CHUNK_SIZE as i64, self);
            if render_chunk.is_empty() {
                render_chunks.remove(&position);
            }
        }
        self.render_chunks = render_chunks;
    }

    pub fn render(
        &self,
        queue: &wgpu::Queue,
        camera_bind_group: &wgpu::BindGroup,
        camera_position: Vector3<f32>,
        render_pass: &mut wgpu::RenderPass<'_>,
    ) {
        for (&position, render_chunk) in &self.render_chunks {
            render_chunk.render(
                queue,
                camera_bind_group,
                &self.chunk_render_pipeline,
                Vector3 {
                    x: (position.x * CHUNK_SIZE as i64) as f32,
                    y: (position.y * CHUNK_SIZE as i64) as f32,
                    z: (position.z * CHUNK_SIZE as i64) as f32,
                },
                camera_position,
                render_pass,
            );
        }
    }
}
