pub struct Game {}

impl Game {
    pub fn new(
        #[expect(unused)] device: &wgpu::Device,
        #[expect(unused)] queue: &wgpu::Queue,
    ) -> Self {
        Self {}
    }

    pub fn update(&mut self, #[expect(unused)] ts: f32) {}

    pub fn render<'a>(
        &'a mut self,
        #[expect(unused)] device: &'a wgpu::Device,
        #[expect(unused)] queue: &'a wgpu::Queue,
        #[expect(unused)] command_encoder: &mut wgpu::CommandEncoder,
        #[expect(unused)] width: u32,
        #[expect(unused)] height: u32,
        #[expect(unused)] dt: f32,
    ) -> impl FnOnce(&mut wgpu::RenderPass<'_>) + use<'a> {
        move |_| {}
    }
}
