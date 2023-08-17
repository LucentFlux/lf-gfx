//! A 'Game' in this context is a program that uses both wgpu and winit.
pub(crate) mod input;
pub(crate) mod window_size;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

use async_trait::async_trait;
use thiserror::Error;
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop, EventLoopWindowTarget},
    window::{Window, WindowBuilder},
};

use crate::LfLimitsExt;

use self::input::InputMap;

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
extern "C" {
    fn alert(s: &str);
}

#[derive(Debug, Error)]
pub enum GameInitialisationFailure {
    #[error("failed to find an adapter (GPU) that supports the render surface")]
    AdapterError,
    #[error("failed to request a device from the adapter chosen: {0}")]
    DeviceError(wgpu::RequestDeviceError),
}

pub enum GameCommand {
    Exit,
}

pub struct GameData {
    pub command_sender: flume::Sender<GameCommand>,
    pub surface: wgpu::Surface,
    pub surface_format: wgpu::TextureFormat,
    pub limits: wgpu::Limits,
    pub config: wgpu::SurfaceConfiguration,
    pub size: winit::dpi::PhysicalSize<u32>,
    pub window: Window,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
}

/// All of the callbacks required to implement a game. This API is built on top of a message passing
/// event system, and so calls to the below methods may be made concurrently, in any order, and on
/// different threads.
#[async_trait]
pub trait Game: Sized {
    /// Data processed before the window exists. This should be minimal and kept to `mpsc` message reception from initialiser threads.
    type InitData;

    type InputType;

    fn target_limits() -> wgpu::Limits {
        wgpu::Limits::downlevel_webgl2_defaults()
    }
    fn default_inputs() -> InputMap<Self::InputType>;

    async fn init(data: &GameData, init: Self::InitData) -> Self;

    fn on_init_failure(error: GameInitialisationFailure) -> ! {
        let error = format!("failed to initialise: {}", error);

        #[cfg(target_arch = "wasm32")]
        {
            alert(&error);
        }

        panic!("{}", error);
    }

    fn window_resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>);

    fn handle_input(&mut self, input: &Self::InputType, activation: input::InputActivation);

    /// Requests that the next frame is drawn into the view, pretty please :)
    fn render_to(&mut self, data: &GameData, view: wgpu::TextureView);

    /// Invoked when the window is told to close (i.e. x pressed, sigint, etc.) but not when
    /// a synthetic exit is triggered by enqueuing `GameCommand::Exit`.
    fn user_exit_requested(&mut self) {}

    /// Invoked right at the end of the program life, after the final frame is rendered.
    fn finished(self) {}
}

/// All the data held by a program/game while running. `T` gives the top-level state for the game
/// implementation
pub(crate) struct GameState<T: Game> {
    data: GameData,
    current_modifiers: input::ModifiersState,
    exit_requested: bool,
    game: T,
    input_map: input::InputMap<T::InputType>,
    command_receiver: flume::Receiver<GameCommand>,
}

impl<T: Game + 'static> GameState<T> {
    // Creating some of the wgpu types requires async code
    async fn new(init: T::InitData, window_target: &EventLoopWindowTarget<()>) -> Self {
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

        let adapter = match instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                force_fallback_adapter: false,
                compatible_surface: Some(&surface),
            })
            .await
        {
            Some(adapter) => adapter,
            None => T::on_init_failure(GameInitialisationFailure::AdapterError),
        };

        let available_limits = if cfg!(target_arch = "wasm32") {
            wgpu::Limits::downlevel_defaults()
        } else {
            adapter.limits()
        };

        let target_limits = T::target_limits();
        let limits = available_limits.intersection(&target_limits);

        let mut features = wgpu::Features::empty();
        // Assume integrated and virtual GPUs, and CPUs, are UMA
        if adapter
            .features()
            .contains(wgpu::Features::MAPPABLE_PRIMARY_BUFFERS)
            && matches!(
                adapter.get_info().device_type,
                wgpu::DeviceType::IntegratedGpu
                    | wgpu::DeviceType::Cpu
                    | wgpu::DeviceType::VirtualGpu
            )
        {
            features |= wgpu::Features::MAPPABLE_PRIMARY_BUFFERS;
        }
        // Things that are always helpful
        features |= adapter.features().intersection(
            wgpu::Features::TIMESTAMP_QUERY | wgpu::Features::TIMESTAMP_QUERY_INSIDE_PASSES,
        );

        let (device, queue) = match adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    features,
                    limits: limits.clone(),
                    label: None,
                },
                None,
            )
            .await
        {
            Ok(vs) => vs,
            Err(e) => T::on_init_failure(GameInitialisationFailure::DeviceError(e)),
        };

        let surface_caps = surface.get_capabilities(&adapter);

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

        let (command_sender, command_receiver) = flume::unbounded();

        let data = GameData {
            command_sender,
            surface,
            surface_format,
            limits,
            config,
            size,
            window,
            device,
            queue,
        };
        let game = T::init(&data, init).await;

        let input_map = T::default_inputs();

        Self {
            data,
            exit_requested: false,
            current_modifiers: input::ModifiersState::default(),
            game,
            command_receiver,
            input_map,
        }
    }

    pub(crate) fn run(init: T::InitData) {
        let event_loop = EventLoop::new();

        // Built on first `Event::Resumed`
        // Taken out on `Event::LoopDestroyed`
        let mut state: Option<Self> = None;
        let mut init = Some(init);

        event_loop.run(move |event, window_target, control_flow| {
            if event == Event::LoopDestroyed {
                state.take().expect("loop is destroyed once").finished();
                return;
            }

            // Resume always emmitted to begin with
            if state.is_none() && event == Event::Resumed {
                state = Some(pollster::block_on(Self::new(
                    init.take().expect("should only initialise once"),
                    window_target,
                )));
            }

            let state = match state.as_mut() {
                None => return,
                Some(state) => state,
            };

            state.process_event(event, window_target, control_flow);
        });
    }

    fn process_event(
        &mut self,
        event: Event<'_, ()>,
        _window_target: &EventLoopWindowTarget<()>,
        control_flow: &mut ControlFlow,
    ) {
        match event {
            Event::WindowEvent {
                ref event,
                window_id,
            } if window_id == self.window().id() => match event {
                WindowEvent::CloseRequested | WindowEvent::Destroyed => self.request_exit(),
                WindowEvent::Resized(physical_size) => {
                    self.resize(*physical_size);
                }
                WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                    self.resize(**new_inner_size);
                }
                WindowEvent::ModifiersChanged(modifiers) => {
                    self.set_current_input_modifiers(modifiers)
                }
                WindowEvent::KeyboardInput {
                    device_id: _device_id,
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
                        self.input(input::InputType::KnownKeyboard(key), activation);
                    } else {
                        eprintln!("unknown key code, scan code: {:?}", input.scancode)
                    }
                }
                _ => {}
            },
            Event::RedrawRequested(window_id) if window_id == self.window().id() => {
                self.data.device.poll(wgpu::MaintainBase::Poll);

                self.pre_frame_update();

                // Check everything the game implementation can send 'upstream'
                if self.exit_requested {
                    *control_flow = ControlFlow::Exit;
                }

                match self.render() {
                    Ok(_) => {}
                    Err(wgpu::SurfaceError::Lost) => self.resize(self.data.size),
                    Err(wgpu::SurfaceError::OutOfMemory) => *control_flow = ControlFlow::Exit,
                    // All other errors (Outdated, Timeout) should be resolved by the next frame
                    Err(e) => eprintln!("{:?}", e),
                }
            }
            Event::MainEventsCleared => {
                self.window().request_redraw();
            }
            _ => {}
        }
    }

    pub fn window(&self) -> &Window {
        &self.data.window
    }

    fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.data.size = new_size;
            self.data.config.width = new_size.width;
            self.data.config.height = new_size.height;
            self.data
                .surface
                .configure(&self.data.device, &self.data.config);

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
        let output = self.data.surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        self.game.render_to(&self.data, view);

        output.present();
        Ok(())
    }

    fn finished(self) {
        self.game.finished()
    }
}
