//! A 'Game' in this context is a program that uses both wgpu and winit.
pub(crate) mod input;
mod surface;
pub(crate) mod window;

use std::sync::{atomic::AtomicBool, Arc};

use log::info;
use thiserror::Error;
use winit::{
    dpi::PhysicalPosition,
    event::{DeviceEvent, Event, WindowEvent},
    event_loop::{EventLoop, EventLoopWindowTarget},
    keyboard::PhysicalKey,
    window::Window,
};

use crate::{game::window::GameWindow, LfLimitsExt};

use self::input::{InputMap, MouseInputType, VectorInputActivation, VectorInputType};

const KEYBINDS_STORAGE_KEY: &'static str = "keybinds";

#[derive(Debug, Error)]
pub enum GameInitialisationFailure {
    #[error("failed to find an adapter (GPU) that supports the render surface")]
    AdapterError,
    #[error("failed to request a device from the adapter chosen: {0}")]
    DeviceError(wgpu::RequestDeviceError),
}

/// A cloneable and distributable flag that can be cheaply queried to see if the game has exited.
///
/// The idea is to clone this into in every thread you spawn so that they can gracefully exit when the game does.
#[derive(Clone)]
pub struct ExitFlag {
    inner: Arc<AtomicBool>,
}

impl ExitFlag {
    fn new() -> Self {
        Self {
            inner: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn get(&self) -> bool {
        self.inner.load(std::sync::atomic::Ordering::SeqCst)
    }

    fn set(&self) {
        self.inner.store(true, std::sync::atomic::Ordering::SeqCst)
    }
}

#[derive(Debug, Clone, Copy)]
pub enum InputMode {
    /// Indicates that any keyboard, mouse or gamepad input should be captured by the input management system,
    /// no raw input events should be passed to the game implementation, and the cursor should be hidden.
    Exclusive,
    /// Indicates that any keyboard, mouse or gamepad input should not be captured by the input management system,
    /// all raw input events should be passed to the game implementation, and the cursor should be shown.
    UI,
    /// Indicates that keyboard, mouse or gamepad input should be captured both by the input management system,
    /// and raw input events should be passed to the game implementation, and the cursor should be shown.
    Unified,
}
impl InputMode {
    fn should_hide_cursor(self) -> bool {
        match self {
            InputMode::Exclusive => true,
            InputMode::UI => false,
            InputMode::Unified => false,
        }
    }
    fn should_handle_input(self) -> bool {
        match self {
            InputMode::Exclusive => true,
            InputMode::UI => false,
            InputMode::Unified => true,
        }
    }
    fn should_propogate_raw_input(self) -> bool {
        match self {
            InputMode::Exclusive => false,
            InputMode::UI => true,
            InputMode::Unified => true,
        }
    }
    fn should_lock_cursor(self) -> bool {
        match self {
            InputMode::Exclusive => true,
            InputMode::UI => false,
            InputMode::Unified => false,
        }
    }
}

/// A command sent to the game to change the game state
pub enum GameCommand {
    Exit,
    SetInputMode(InputMode),
    SetMouseSensitivity(f32),
}

pub struct GameData {
    pub command_sender: flume::Sender<GameCommand>,
    pub surface_format: wgpu::TextureFormat,
    pub limits: wgpu::Limits,
    pub size: winit::dpi::PhysicalSize<u32>,
    pub window: GameWindow,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub exit_flag: ExitFlag,
}

/// All of the callbacks required to implement a game. This API is built on top of a message passing
/// event system, and so calls to the below methods may be made concurrently, in any order, and on
/// different threads.
pub trait Game: Sized {
    /// Data processed before the window exists. This should be minimal and kept to `mpsc` message reception from initialiser threads.
    type InitData;

    type LinearInputType: serde::Serialize + for<'a> serde::Deserialize<'a>;
    type VectorInputType: serde::Serialize + for<'a> serde::Deserialize<'a>;

    fn title() -> impl Into<String>;

    fn target_limits() -> wgpu::Limits {
        wgpu::Limits::downlevel_webgl2_defaults()
    }
    fn default_inputs(&self) -> InputMap<Self::LinearInputType, Self::VectorInputType>;

    fn init(data: &GameData, init: Self::InitData) -> Self;

    fn on_init_failure(error: GameInitialisationFailure) -> ! {
        let error = format!("failed to initialise: {}", error);

        panic!("{}", error);
    }

    /// Allows you to intercept and cancel events, before passing them off to the standard event handler,
    /// to allow for egui integration, among others.
    ///
    /// This method only receives input events if the cursor is not captured, to avoid UI glitches.
    fn process_raw_event<'a, T>(&mut self, _: &GameData, event: Event<T>) -> Option<Event<T>> {
        Some(event)
    }

    fn window_resize(&mut self, data: &GameData, new_size: winit::dpi::PhysicalSize<u32>);

    fn handle_linear_input(
        &mut self,
        data: &GameData,
        input: &Self::LinearInputType,
        activation: input::LinearInputActivation,
    );

    fn handle_vector_input(
        &mut self,
        data: &GameData,
        input: &Self::VectorInputType,
        activation: input::VectorInputActivation,
    );

    /// Requests that the next frame is drawn into the view, pretty please :)
    fn render_to(&mut self, data: &GameData, view: wgpu::TextureView);

    /// Invoked when the window is told to close (i.e. x pressed, sigint, etc.) but not when
    /// a synthetic exit is triggered by enqueuing `GameCommand::Exit`. To actually do something with the
    /// user's request to quit, this method must enqueue `GameCommand::Exit`
    fn user_exit_requested(&mut self, data: &GameData) {
        let _ = data.command_sender.send(GameCommand::Exit);
    }

    /// Invoked right at the end of the program life, after the final frame is rendered.
    fn finished(self, _: GameData) {}
}

/// All the data held by a program/game while running. `T` gives the top-level state for the game
/// implementation
pub(crate) struct GameState<T: Game> {
    data: GameData,
    game: T,
    input_map: input::InputMap<T::LinearInputType, T::VectorInputType>,
    command_receiver: flume::Receiver<GameCommand>,

    surface: surface::ResizableSurface,

    // While true, disallows cursor movement
    input_mode: InputMode,
    // The last position we saw the cursor at
    last_cursor_position: PhysicalPosition<f64>,
    // A multiplier, from pixels moved to intensity, clamped at 1.0
    mouse_sensitivity: f32,
}

impl<T: Game + 'static> GameState<T> {
    // Creating some of the wgpu types requires async code
    async fn new(init: T::InitData, window: GameWindow) -> Self {
        let size = (&window).inner_size();

        #[cfg(debug_assertions)]
        let flags = wgpu::InstanceFlags::DEBUG | wgpu::InstanceFlags::VALIDATION;
        #[cfg(not(debug_assertions))]
        let flags = wgpu::InstanceFlags::DISCARD_HAL_LABELS;

        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::util::backend_bits_from_env().unwrap_or(wgpu::Backends::all()),
            dx12_shader_compiler: wgpu::util::dx12_shader_compiler_from_env().unwrap_or_default(),
            flags,
            gles_minor_version: wgpu::Gles3MinorVersion::Automatic,
        });

        let surface;
        // On wasm, we need to insert ourselves into the DOM
        #[cfg(target_arch = "wasm32")]
        {
            surface = instance
                .create_surface_from_canvas(window.canvas())
                .unwrap();
        }
        // # Safety
        //
        // The surface needs to live as long as the window that created it.
        // State owns the window so this should be safe.
        #[cfg(not(target_arch = "wasm32"))]
        {
            surface = unsafe { instance.create_surface(&*window) }.unwrap();
        }

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
            wgpu::Limits::downlevel_webgl2_defaults()
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

        info!("info: {:#?}", adapter.get_info());
        info!("limits: {:#?}", adapter.limits());

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

        // Configure surface
        let mut surface_config = surface
            .get_default_config(&adapter, size.width, size.height)
            .expect("got an adapter based on the surface");
        surface_config.present_mode = wgpu::PresentMode::AutoVsync;
        surface.configure(&device, &surface_config);

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
        let surface = surface::ResizableSurface::new(surface, &device, config);

        let (command_sender, command_receiver) = flume::unbounded();

        // Some state can be set by commands to ensure valid initial state.
        command_sender
            .try_send(GameCommand::SetInputMode(InputMode::Unified))
            .expect("unbounded queue held by this thread should send immediately");

        let data = GameData {
            command_sender,
            surface_format,
            limits,
            size,
            window,
            device,
            queue,
            exit_flag: ExitFlag::new(),
        };
        let game = T::init(&data, init);

        // Gather inputs as a combination of registered user preferences and defaults.
        let mut input_map = game.default_inputs();
        let user_preferences = crate::local_storage::load(KEYBINDS_STORAGE_KEY);
        if let Some(user_preferences) = user_preferences {
            if let Ok(user_preferences) = InputMap::deserialize(&user_preferences) {
                input_map.union(user_preferences);
            }
        }
        if let Err(err) = crate::local_storage::store(
            KEYBINDS_STORAGE_KEY,
            &serde_json::to_string_pretty(&input_map.serialize()).expect("keys serializable"),
        ) {
            crate::alert_dialogue(&format!("Failed to access storage:\n{err}"));
            panic!("game requires storage access");
        }

        Self {
            data,
            game,
            surface,
            command_receiver,
            input_map,
            input_mode: InputMode::Unified,
            last_cursor_position: PhysicalPosition { x: 0.0, y: 0.0 },
            mouse_sensitivity: 0.01,
        }
    }

    pub(crate) fn run(init: T::InitData) {
        let event_loop = EventLoop::new().expect("could not create game loop");

        // Built on first `Event::Resumed`
        // Taken out on `Event::LoopDestroyed`
        let mut state: Option<Self> = None;
        let (state_transmission, state_reception) = flume::bounded(1);
        let mut init = Some((init, state_transmission));

        event_loop
            .run(move |event, window_target| {
                if event == Event::LoopExiting {
                    state.take().expect("loop is destroyed once").finished();
                    return;
                }

                // Resume always emmitted to begin with - use it to begin an async method to create the game state.
                if state.is_none() && event == Event::Resumed {
                    if let Some((init, state_transmission)) = init.take() {
                        async fn build_state<T: Game + 'static>(
                            init: T::InitData,
                            window: GameWindow,
                            state_transmission: flume::Sender<GameState<T>>,
                        ) {
                            let state = GameState::<T>::new(init, window).await;
                            state_transmission.try_send(state).unwrap();
                        }

                        let window = GameWindow::new::<T>(window_target);
                        crate::block_on(build_state::<T>(init, window, state_transmission));
                    }
                }

                // On any future events, check if the game state has been created and receive it.
                let state = match state.as_mut() {
                    None => {
                        if let Ok(new_state) = state_reception.try_recv() {
                            state = Some(new_state);
                            state.as_mut().unwrap()
                        } else {
                            return;
                        }
                    }
                    Some(state) => state,
                };

                state.receive_event(event, window_target);
            })
            .expect("run err");
    }

    fn is_input_event(event: &Event<()>) -> bool {
        match event {
            winit::event::Event::WindowEvent { event, .. } => match event {
                WindowEvent::CursorMoved { .. }
                | WindowEvent::CursorEntered { .. }
                | WindowEvent::CursorLeft { .. }
                | WindowEvent::MouseWheel { .. }
                | WindowEvent::MouseInput { .. }
                | WindowEvent::TouchpadRotate { .. }
                | WindowEvent::TouchpadPressure { .. }
                | WindowEvent::AxisMotion { .. }
                | WindowEvent::Touch(_)
                | WindowEvent::KeyboardInput { .. }
                | WindowEvent::ModifiersChanged(_)
                | WindowEvent::Ime(_)
                | WindowEvent::TouchpadMagnify { .. }
                | WindowEvent::SmartMagnify { .. } => true,
                _ => false,
            },
            winit::event::Event::DeviceEvent { event, .. } => match event {
                DeviceEvent::MouseMotion { .. }
                | DeviceEvent::MouseWheel { .. }
                | DeviceEvent::Motion { .. }
                | DeviceEvent::Button { .. }
                | DeviceEvent::Key(_) => true,
                _ => false,
            },
            _ => false,
        }
    }

    fn receive_event(&mut self, mut event: Event<()>, window_target: &EventLoopWindowTarget<()>) {
        // Discard events that aren't for us
        event = match event {
            Event::WindowEvent { window_id, .. } if window_id != self.window().id() => return,
            event => event,
        };

        // We filter all window events through the game to allow it to integrate with other libraries, such as egui.
        // But only send keyboard and mouse input events to UI if the mouse isn't captured.
        let should_send_input = self.input_mode.should_propogate_raw_input();
        if should_send_input || !Self::is_input_event(&event) {
            event = match self.game.process_raw_event(&self.data, event) {
                None => return,
                Some(event) => event,
            };
        }

        self.process_event(event, window_target)
    }

    fn process_event(&mut self, event: Event<()>, window_target: &EventLoopWindowTarget<()>) {
        match event {
            Event::WindowEvent { event, window_id } if window_id == self.window().id() => {
                match event {
                    WindowEvent::CloseRequested | WindowEvent::Destroyed => self.request_exit(),
                    // (0, 0) means minimized on Windows.
                    WindowEvent::Resized(winit::dpi::PhysicalSize {
                        width: 0,
                        height: 0,
                    }) => {}
                    WindowEvent::Resized(physical_size) => {
                        log::debug!("Resized: {:?}", physical_size);
                        self.resize(physical_size);
                    }
                    WindowEvent::ScaleFactorChanged { scale_factor, .. } => {
                        log::debug!("Scale Factor Changed: {:?}", scale_factor);
                        //self.resize(*new_inner_size);
                    }
                    WindowEvent::KeyboardInput {
                        device_id: _device_id,
                        event,
                        is_synthetic,
                    } if !is_synthetic && !event.repeat => {
                        if let PhysicalKey::Code(key) = event.physical_key {
                            let activation = match event.state {
                                winit::event::ElementState::Pressed => 1.0,
                                winit::event::ElementState::Released => 0.0,
                            };
                            let activation = input::LinearInputActivation::try_from(activation)
                                .expect("from const");
                            self.linear_input(
                                input::LinearInputType::KnownKeyboard(key.into()),
                                activation,
                            );
                        } else {
                            eprintln!("unknown key code, scan code: {:?}", event.physical_key)
                        }
                    }
                    WindowEvent::CursorMoved {
                        device_id: _device_id,
                        position,
                        ..
                    } => {
                        let delta_x = position.x - self.last_cursor_position.x;
                        let delta_y = position.y - self.last_cursor_position.y;

                        // Only trigger a single linear event, depending on the largest movement
                        if delta_x.abs() > 2.0 || delta_y.abs() > 2.0 {
                            self.process_linear_mouse_movement(delta_x, delta_y);
                        }

                        // Also trigger a vector input
                        self.vector_input(
                            VectorInputType::MouseMove,
                            VectorInputActivation::clamp(
                                delta_x as f32 * self.mouse_sensitivity,
                                delta_y as f32 * self.mouse_sensitivity,
                            ),
                        );

                        self.last_cursor_position = position.cast();

                        // Winit doesn't support cursor locking on a lot of platforms, so do it manually.
                        let should_lock_cursor = self.input_mode.should_lock_cursor();
                        if should_lock_cursor {
                            let mut center = self.data.window.inner_size();
                            center.width /= 2;
                            center.height /= 2;

                            let old_pos = position.cast::<u32>();
                            let new_pos = PhysicalPosition::new(center.width, center.height);

                            if old_pos != new_pos {
                                // Ignore result - if it doesn't work then there's not much we can do.
                                let _ = self.data.window.set_cursor_position(new_pos);
                            }

                            self.last_cursor_position = new_pos.cast();
                        }
                    }
                    WindowEvent::RedrawRequested => {
                        self.data.device.poll(wgpu::MaintainBase::Poll);

                        self.pre_frame_update();

                        // Check everything the game implementation can send to us
                        if self.data.exit_flag.get() {
                            window_target.exit();
                        }

                        let res = self.render();
                        match res {
                            Ok(_) => {}
                            Err(wgpu::SurfaceError::Lost) => self.resize(self.data.size),
                            Err(wgpu::SurfaceError::OutOfMemory) => {
                                window_target.exit();
                            }
                            // All other errors (Outdated, Timeout) should be resolved by the next frame
                            Err(e) => eprintln!("{:?}", e),
                        }
                    }
                    _ => {}
                }
            }
            Event::DeviceEvent { device_id, event } => {
                log::debug!("device event: {device_id:?}::{event:?}");
            }
            Event::AboutToWait => {
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

            self.surface.resize(new_size, &self.data.queue);

            self.game.window_resize(&self.data, new_size)
        }
    }

    fn process_linear_mouse_movement(&mut self, delta_x: f64, delta_y: f64) {
        if delta_x.abs() > delta_y.abs() {
            if delta_x > 0.0 {
                self.linear_input(
                    input::LinearInputType::Mouse(MouseInputType::MoveRight),
                    input::LinearInputActivation::clamp(delta_x as f32 * self.mouse_sensitivity),
                );
            } else {
                self.linear_input(
                    input::LinearInputType::Mouse(MouseInputType::MoveLeft),
                    input::LinearInputActivation::clamp(-delta_x as f32 * self.mouse_sensitivity),
                );
            }
        } else {
            if delta_y > 0.0 {
                self.linear_input(
                    input::LinearInputType::Mouse(MouseInputType::MoveUp),
                    input::LinearInputActivation::clamp(delta_y as f32 * self.mouse_sensitivity),
                );
            } else {
                self.linear_input(
                    input::LinearInputType::Mouse(MouseInputType::MoveDown),
                    input::LinearInputActivation::clamp(-delta_y as f32 * self.mouse_sensitivity),
                );
            }
        }
    }

    fn linear_input(
        &mut self,
        inputted: input::LinearInputType,
        activation: input::LinearInputActivation,
    ) {
        if !self.input_mode.should_handle_input() {
            return;
        }
        let input_value = self.input_map.get_linear(inputted);
        if let Some(input_value) = input_value {
            self.game
                .handle_linear_input(&self.data, input_value, activation)
        }
    }

    fn vector_input(
        &mut self,
        inputted: input::VectorInputType,
        activation: input::VectorInputActivation,
    ) {
        if !self.input_mode.should_handle_input() {
            return;
        }
        let input_value = self.input_map.get_vector(inputted);
        if let Some(input_value) = input_value {
            self.game
                .handle_vector_input(&self.data, input_value, activation)
        }
    }

    fn request_exit(&mut self) {
        self.data.exit_flag.set();
        self.game.user_exit_requested(&self.data);
    }

    fn pre_frame_update(&mut self) {
        // Get all the things the game wants to do before the next frame
        while let Ok(cmd) = self.command_receiver.try_recv() {
            match cmd {
                GameCommand::Exit => self.data.exit_flag.set(),
                GameCommand::SetInputMode(input_mode) => {
                    self.input_mode = input_mode;

                    let should_show_cursor = !input_mode.should_hide_cursor();
                    self.data.window.set_cursor_visible(should_show_cursor);
                }
                GameCommand::SetMouseSensitivity(new_sensitivity) => {
                    self.mouse_sensitivity = new_sensitivity;
                }
            }
        }
    }

    fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        // If we are in the process of resizing, don't do anything
        if let Some(surface) = self.surface.get(&self.data.device) {
            let was_suboptimal = {
                let output = surface.get_current_texture()?;
                let view = output
                    .texture
                    .create_view(&wgpu::TextureViewDescriptor::default());

                self.game.render_to(&self.data, view);

                let was_suboptimal = output.suboptimal;

                output.present();

                was_suboptimal
            };

            if was_suboptimal {
                // Force recreation
                return Err(wgpu::SurfaceError::Lost);
            }
        }
        Ok(())
    }

    fn finished(self) {
        self.game.finished(self.data)
    }
}
