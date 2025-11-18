use bytemuck::NoUninit;
use std::marker::PhantomData;

pub struct StorageBuffer<T> {
    label: &'static str,
    buffer: wgpu::Buffer,
    _data: PhantomData<T>,
}

impl<T> StorageBuffer<T>
where
    T: NoUninit,
{
    pub fn new(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        label: &'static str,
        data: &[T],
    ) -> Self {
        let buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some(label),
            size: wgpu::BufferAddress::try_from(size_of_val(data).max(size_of::<T>()))
                .expect("the size of data should fit in a u64"),
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        queue.write_buffer(&buffer, 0, bytemuck::cast_slice(data));
        Self {
            label,
            buffer,
            _data: PhantomData,
        }
    }

    #[must_use]
    pub fn write(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, data: &[T]) -> bool {
        let size = wgpu::BufferAddress::try_from(size_of_val(data).max(size_of::<T>()))
            .expect("the size of data should fit in a u64");

        let reallocated = size > self.buffer.size() || size < self.buffer.size() / 2;
        if reallocated {
            self.buffer = device.create_buffer(&wgpu::BufferDescriptor {
                label: Some(self.label),
                size,
                usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
        }

        queue.write_buffer(&self.buffer, 0, bytemuck::cast_slice(data));

        reallocated
    }

    pub fn buffer(&self) -> &wgpu::Buffer {
        &self.buffer
    }
}
