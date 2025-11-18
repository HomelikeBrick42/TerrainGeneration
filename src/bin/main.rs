use std::{
    sync::Arc,
    time::{Duration, Instant},
};

use blocks::game::Game;
use winit::{
    application::ApplicationHandler,
    dpi::PhysicalSize,
    error::EventLoopError,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, EventLoop},
    window::{Window, WindowAttributes, WindowId},
};

struct WindowState {
    window: Arc<Window>,
    surface: wgpu::Surface<'static>,
    surface_config: wgpu::SurfaceConfiguration,
}

struct App {
    last_time: Option<Instant>,
    dt: Duration,
    game: Game,
    instance: wgpu::Instance,
    #[expect(unused)]
    adapter: wgpu::Adapter,
    device: wgpu::Device,
    queue: wgpu::Queue,
    window_state: Option<WindowState>,
}

impl ApplicationHandler for App {
    fn suspended(&mut self, #[expect(unused)] event_loop: &ActiveEventLoop) {
        self.window_state = None;
        self.last_time = None;
    }

    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        self.window_state = None;

        let window = Arc::new(
            event_loop
                .create_window(WindowAttributes::default().with_title("Blocks"))
                .expect("the window should get created"),
        );

        let surface = self
            .instance
            .create_surface(window.clone())
            .expect("the surface should get created");

        let PhysicalSize { width, height } = window.inner_size();
        let surface_config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: wgpu::TextureFormat::Bgra8Unorm,
            width,
            height,
            present_mode: wgpu::PresentMode::AutoNoVsync,
            desired_maximum_frame_latency: 2,
            alpha_mode: wgpu::CompositeAlphaMode::Opaque,
            view_formats: vec![],
        };
        surface.configure(&self.device, &surface_config);

        self.window_state = Some(WindowState {
            window,
            surface,
            surface_config,
        });
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: WindowId,
        event: WindowEvent,
    ) {
        let Some(WindowState { ref window, .. }) = self.window_state else {
            return;
        };
        if window.id() != window_id {
            return;
        }

        match event {
            WindowEvent::Resized(_) => {
                self.resize();
            }

            WindowEvent::CloseRequested | WindowEvent::Destroyed => {
                event_loop.exit();
            }

            _ => {}
        }
    }

    fn new_events(
        &mut self,
        #[expect(unused)] event_loop: &ActiveEventLoop,
        #[expect(unused)] cause: winit::event::StartCause,
    ) {
        let time = Instant::now();
        self.dt = time - self.last_time.unwrap_or(time);
        self.last_time = Some(time);
    }

    fn about_to_wait(&mut self, #[expect(unused)] event_loop: &ActiveEventLoop) {
        self.game.update(self.dt.as_secs_f32());
        self.render();
    }

    fn exiting(&mut self, #[expect(unused)] event_loop: &ActiveEventLoop) {
        self.window_state = None;
    }
}

impl App {
    pub fn resize(&mut self) {
        let Some(WindowState {
            ref window,
            ref surface,
            ref mut surface_config,
        }) = self.window_state
        else {
            return;
        };

        let PhysicalSize { width, height } = window.inner_size();
        if width > 0 && height > 0 {
            surface_config.width = width;
            surface_config.height = height;
            surface.configure(&self.device, surface_config);
        }
    }

    pub fn render(&mut self) {
        let Some(WindowState {
            ref window,
            ref surface,
            ..
        }) = self.window_state
        else {
            return;
        };
        let PhysicalSize { width, height } = window.inner_size();

        let surface_texture = match surface.get_current_texture() {
            Ok(surface_texture) => surface_texture,
            Err(wgpu::SurfaceError::Timeout) => return,
            Err(wgpu::SurfaceError::Outdated | wgpu::SurfaceError::Lost) => {
                self.resize();
                return;
            }
            error => error.expect("expected to get the next surface image"),
        };
        let surface_texture_view = surface_texture
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        {
            let mut command_encoder =
                self.device
                    .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                        label: Some("Render Encoder"),
                    });

            let render = self.game.render(
                &self.device,
                &self.queue,
                &mut command_encoder,
                width,
                height,
                self.dt.as_secs_f32(),
            );

            {
                let mut render_pass =
                    command_encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                        label: Some("Render Pass"),
                        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                            view: &surface_texture_view,
                            depth_slice: None,
                            resolve_target: None,
                            ops: wgpu::Operations {
                                load: wgpu::LoadOp::Clear(wgpu::Color {
                                    r: 0.0,
                                    g: 0.0,
                                    b: 0.0,
                                    a: 1.0,
                                }),
                                store: wgpu::StoreOp::Store,
                            },
                        })],
                        depth_stencil_attachment: None,
                        timestamp_writes: None,
                        occlusion_query_set: None,
                    });

                render(&mut render_pass);
            }

            self.queue.submit(std::iter::once(command_encoder.finish()));
        }

        window.pre_present_notify();
        surface_texture.present();
    }
}

fn main() -> Result<(), EventLoopError> {
    let event_loop = EventLoop::new()?;

    let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
        backends: wgpu::Backends::all(),
        flags: wgpu::InstanceFlags::from_build_config().with_env(),
        memory_budget_thresholds: wgpu::MemoryBudgetThresholds::default(),
        backend_options: wgpu::BackendOptions::from_env_or_default(),
    });

    let (adapter, device, queue) = pollster::block_on(async {
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptionsBase {
                power_preference: wgpu::PowerPreference::HighPerformance,
                force_fallback_adapter: false,
                compatible_surface: None,
            })
            .await
            .expect("the adapter should get created");

        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                label: Some("wgpu device"),
                required_features: wgpu::Features::empty(),
                required_limits: adapter.limits(),
                experimental_features: wgpu::ExperimentalFeatures::disabled(),
                memory_hints: wgpu::MemoryHints::Performance,
                trace: wgpu::Trace::Off,
            })
            .await
            .expect("device and queue should get created");

        (adapter, device, queue)
    });

    let mut app = App {
        last_time: None,
        dt: Duration::ZERO,
        game: Game::new(&device, &queue),
        instance,
        adapter,
        device,
        queue,
        window_state: None,
    };

    event_loop.run_app(&mut app)
}
