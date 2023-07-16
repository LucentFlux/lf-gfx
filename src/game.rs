//! A 'Game' in this context is a program that uses both wgpu and winit.
pub(crate) mod input;
pub(crate) mod window_size;

use async_trait::async_trait;
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop, EventLoopWindowTarget},
    window::{Window, WindowBuilder},
};

use crate::{InstanceExt, LimitsExt};

use self::input::InputMap;

pub enum GameCommand {
    Exit,
}

pub struct GameInitData<'a> {
    pub command_sender: tokio::sync::mpsc::UnboundedSender<GameCommand>,
    pub surface: &'a wgpu::Surface,
    pub limits: &'a wgpu::Limits,
    pub config: &'a wgpu::SurfaceConfiguration,
    pub size: winit::dpi::PhysicalSize<u32>,
    pub window: &'a Window,
}

/// All of the callbacks required to implement a game. This API is built on top of a message passing
/// event system, and so calls to the below methods may be made concurrently, in any order, and on
/// different threads.
#[async_trait]
pub trait Game: Sized {
    type InputType;

    fn target_limits() -> wgpu::Limits {
        wgpu::Limits::downlevel_webgl2_defaults()
    }
    fn default_inputs() -> InputMap<Self::InputType>;

    async fn init(data: GameInitData<'_>) -> Self;

    fn window_resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>);

    fn handle_input(&mut self, input: &Self::InputType, activation: input::InputActivation);

    /// Requests that the next frame is drawn into the view, pretty please :)
    fn render_to(&mut self, view: wgpu::TextureView);

    /// Invoked when the window is told to close (i.e. x pressed, sigint, etc.) but not when
    /// a synthetic exit is triggered by enqueuing `GameCommand::Exit`.
    fn user_exit_requested(&mut self) {}

    /// Invoked right at the end of the program life, after the final frame is rendered.
    fn finished(self) {}
}

/// All the data held by a program/game while running. `T` gives the top-level state for the game
/// implementation
pub(crate) struct GameState<T: Game> {
    surface: wgpu::Surface,
    device: wgpu::Device,
    queue: wgpu::Queue,
    limits: wgpu::Limits,
    config: wgpu::SurfaceConfiguration,
    size: winit::dpi::PhysicalSize<u32>,
    window: Window,
    current_modifiers: input::ModifiersState,
    exit_requested: bool,
    game: T,
    input_map: input::InputMap<T::InputType>,
    command_receiver: tokio::sync::mpsc::UnboundedReceiver<GameCommand>,
}

impl<T: Game + 'static> GameState<T> {
    // Creating some of the wgpu types requires async code
    async fn new(window_target: &EventLoopWindowTarget<()>) -> Self {
        let window = WindowBuilder::new().build(window_target).unwrap();

        let size = window.inner_size();

        // The instance is a handle to our GPU
        // Backends::all => Vulkan + Metal + DX12 + Browser WebGPU
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::util::backend_bits_from_env().unwrap_or(wgpu::Backends::all()),
            dx12_shader_compiler: wgpu::util::dx12_shader_compiler_from_env().unwrap_or_default(),
        });

        // # Safety
        //
        // The surface needs to live as long as the window that created it.
        // State owns the window so this should be safe.
        let surface = unsafe { instance.create_surface(&window) }.unwrap();

        let adapter = instance
            .request_powerful_adapter(
                wgpu::Backends::all(),
                crate::AdapterQuery {
                    compatible_surface: Some(&surface),
                    physical_blacklist: &[],
                    force_adapter_type: None,
                },
            )
            .unwrap();

        let available_limits = if cfg!(target_arch = "wasm32") {
            wgpu::Limits::downlevel_defaults()
        } else {
            adapter.limits()
        };

        let target_limits = T::target_limits();
        let limits = available_limits.intersection(&target_limits);

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    features: wgpu::Features::empty(),
                    limits: limits.clone(),
                    label: None,
                },
                None, // Trace path
            )
            .await
            .unwrap();

        let surface_caps = surface.get_capabilities(&adapter);
        // Shader code in this tutorial assumes an sRGB surface texture. Using a different
        // one will result all the colors coming out darker. If you want to support non
        // sRGB surfaces, you'll need to account for that when drawing to the frame.
        let surface_format = surface_caps
            .formats
            .iter()
            .copied()
            .filter(|f| f.is_srgb())
            .next()
            .unwrap_or(surface_caps.formats[0]);
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
        };
        surface.configure(&device, &config);

        let (command_sender, command_receiver) = tokio::sync::mpsc::unbounded_channel();

        let init_data = GameInitData {
            command_sender,
            surface: &surface,
            limits: &limits,
            config: &config,
            size,
            window: &window,
        };
        let game = T::init(init_data).await;

        let input_map = T::default_inputs();

        Self {
            exit_requested: false,
            current_modifiers: input::ModifiersState::default(),
            window,
            surface,
            device,
            queue,
            limits,
            config,
            size,
            game,
            command_receiver,
            input_map,
        }
    }

    pub(crate) fn run() {
        let event_loop = EventLoop::new();

        // Built on first `Event::Resumed`
        // Taken out on `Event::LoopDestroyed`
        let mut state: Option<Self> = None;

        event_loop.run(move |event, window_target, control_flow| {
            if event == Event::LoopDestroyed {
                state.take().expect("loop is destroyed once").finished();
                return;
            }
            // Resume always emmitted to begin with
            if state.is_none() && event == Event::Resumed {
                state = Some(pollster::block_on(Self::new(window_target)));
            }

            let state = state
                .as_mut()
                .expect("state is built at start and removed on finish");
            match event {
                Event::WindowEvent {
                    ref event,
                    window_id,
                } if window_id == state.window().id() => match event {
                    WindowEvent::CloseRequested | WindowEvent::Destroyed => state.request_exit(),
                    WindowEvent::Resized(physical_size) => {
                        state.resize(*physical_size);
                    }
                    WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                        state.resize(**new_inner_size);
                    }
                    WindowEvent::ModifiersChanged(modifiers) => {
                        state.set_current_input_modifiers(modifiers)
                    }
                    WindowEvent::KeyboardInput {
                        device_id,
                        input,
                        is_synthetic,
                    } if !*is_synthetic => {
                        if let Some(key) = input.virtual_keycode {
                            let activation = match input.state {
                                winit::event::ElementState::Pressed => 1.0,
                                winit::event::ElementState::Released => 0.0,
                            };
                            let activation =
                                input::InputActivation::try_from(activation).expect("from const");
                            state.input(input::InputType::KnownKeyboard(key), activation);
                        } else {
                            eprintln!("unknown key code, scan code: {:?}", input.scancode)
                        }
                    }
                    _ => {}
                },
                Event::RedrawRequested(window_id) if window_id == state.window().id() => {
                    state.pre_frame_update();

                    // Check everything the game implementation can send 'upstream'
                    if state.exit_requested {
                        *control_flow = ControlFlow::Exit;
                    }

                    match state.render() {
                        Ok(_) => {}
                        Err(wgpu::SurfaceError::Lost) => state.resize(state.size),
                        Err(wgpu::SurfaceError::OutOfMemory) => *control_flow = ControlFlow::Exit,
                        // All other errors (Outdated, Timeout) should be resolved by the next frame
                        Err(e) => eprintln!("{:?}", e),
                    }
                }
                Event::MainEventsCleared => {
                    state.window().request_redraw();
                }
                _ => {}
            }
        });
    }

    pub fn window(&self) -> &Window {
        &self.window
    }
}

impl<T: Game + 'static> GameState<T> {
    fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);

            self.game.window_resize(new_size)
        }
    }

    fn set_current_input_modifiers(&mut self, modifiers: &winit::event::ModifiersState) {
        self.current_modifiers = input::ModifiersState {
            shift: modifiers.shift(),
            ctrl: modifiers.ctrl(),
            alt: modifiers.alt(),
            logo: modifiers.logo(),
        }
    }

    fn input(&mut self, inputted: input::InputType, activation: input::InputActivation) {
        let code = input::InputCode {
            modifiers: self.current_modifiers,
            inputted,
        };
        let input_value = self.input_map.get(&code);
        if let Some(input_value) = input_value {
            self.game.handle_input(input_value, activation)
        }
    }

    fn request_exit(&mut self) {
        self.game.user_exit_requested();
        self.exit_requested = true;
    }

    fn pre_frame_update(&mut self) {
        // Get all the things the game wants to do before the next frame
        while let Ok(cmd) = self.command_receiver.try_recv() {
            match cmd {
                GameCommand::Exit => self.exit_requested = true,
            }
        }
    }

    fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        let output = self.surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        self.game.render_to(view);

        output.present();
        Ok(())
    }

    fn finished(self) {
        self.game.finished()
    }
}
