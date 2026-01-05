use std::sync::{Arc, atomic::AtomicBool};

use gilrs::Gilrs;
use wgpu::{
    Device, DeviceDescriptor, PollType, Queue, RequestAdapterOptions, Surface,
    SurfaceConfiguration, TextureFormat, TextureViewDescriptor,
};
use winit::{
    application::ApplicationHandler,
    event::{KeyEvent, WindowEvent},
    event_loop::{ActiveEventLoop, EventLoop},
    keyboard::{KeyCode, PhysicalKey},
    window::{Window, WindowId},
};

use crate::game::Game;

#[derive(Debug, Clone, Copy)]
pub struct GameContext<'a> {
    pub window: &'a Window,
    pub device: &'a Device,
    pub queue: &'a Queue,
    pub surface_format: TextureFormat,
    should_exit: Option<&'a AtomicBool>,
}

#[derive(Debug)]
pub enum GameEvent {
    CloseRequested,
    Key { code: KeyCode, is_held: bool },
    Button { code: ButtonCode, value: f32 },
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ButtonCode {
    LeftStickRight,
    LeftStickLeft,
    LeftStickUp,
    LeftStickDown,
    RightStickRight,
    RightStickLeft,
    RightStickUp,
    RightStickDown,
    DPadUp,
    DPadDown,
    DPadLeft,
    DPadRight,
    South,
    East,
    North,
    West,
    LeftTrigger,
    LeftTrigger2,
    RightTrigger,
    RightTrigger2,
    Start,
    Select,
    LeftThumb,
    RightThumb,
    C,
    Z,
    Mode,
    Unknown,
}

pub fn run() {
    let event_loop = EventLoop::new().expect("failed to create event loop");
    let mut runner = Runner::Uninit;
    event_loop.run_app(&mut runner).expect("failed to run app");
}

enum Runner {
    Uninit,
    Init(InitRunner),
}

struct InitRunner {
    window: Arc<Window>,
    device: Device,
    queue: Queue,
    surface: Surface<'static>,
    surface_config: SurfaceConfiguration,
    gilrs: Gilrs,
    game: Game,
}

impl<'a> GameContext<'a> {
    pub fn exit(&self) {
        if let Some(should_exit) = self.should_exit {
            should_exit.store(true, std::sync::atomic::Ordering::Relaxed);
        } else {
            panic!("cannot exit the game from this context");
        }
    }
}

impl ApplicationHandler for Runner {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if matches!(self, Runner::Uninit) {
            *self = Runner::Init(InitRunner::new(event_loop));
        }
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _: WindowId, event: WindowEvent) {
        if let Runner::Init(runner) = self {
            runner.window_event(event_loop, event);
        }
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        if let Runner::Init(runner) = self {
            runner.about_to_wait(event_loop);
        }
    }
}

impl InitRunner {
    fn new(event_loop: &ActiveEventLoop) -> Self {
        let window = {
            let window = event_loop
                .create_window(Game::window_attributes())
                .expect("failed to create window");

            Arc::new(window)
        };

        let device;
        let queue;
        let surface;
        let surface_config;
        {
            let instance = wgpu::Instance::default();

            let adapter = instance.request_adapter(&RequestAdapterOptions::default());
            let adapter = pollster::block_on(adapter).expect("failed to get adapter");

            let device_queue = adapter.request_device(&DeviceDescriptor::default());
            let device_queue = pollster::block_on(device_queue).expect("failed to get device");
            device = device_queue.0;
            queue = device_queue.1;

            surface = instance
                .create_surface(window.clone())
                .expect("failed to create surface");

            surface_config = surface
                .get_default_config(
                    &adapter,
                    window.inner_size().width,
                    window.inner_size().height,
                )
                .expect("failed to create surface config");

            surface.configure(&device, &surface_config);
        };

        let gilrs = Gilrs::new().expect("failed to create gilrs");

        let game = Game::new(GameContext {
            window: &window,
            device: &device,
            queue: &queue,
            surface_format: surface_config.format,
            should_exit: None,
        });

        Self {
            window,
            device,
            queue,
            surface,
            surface_config,
            gilrs,
            game,
        }
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, event: WindowEvent) {
        let game_event = match &event {
            WindowEvent::CloseRequested => Some(GameEvent::CloseRequested),

            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        physical_key: PhysicalKey::Code(code),
                        state,
                        ..
                    },
                ..
            } => Some(GameEvent::Key {
                code: *code,
                is_held: state.is_pressed(),
            }),

            _ => None,
        };

        if let Some(game_event) = &game_event {
            let should_exit = AtomicBool::new(false);

            self.game.event(
                game_event,
                GameContext {
                    window: &self.window,
                    device: &self.device,
                    queue: &self.queue,
                    surface_format: self.surface_config.format,
                    should_exit: Some(&should_exit),
                },
            );

            if should_exit.load(std::sync::atomic::Ordering::Relaxed) {
                self.game.end(GameContext {
                    window: &self.window,
                    device: &self.device,
                    queue: &self.queue,
                    surface_format: self.surface_config.format,
                    should_exit: None,
                });

                event_loop.exit();
                return;
            }
        }

        match &event {
            WindowEvent::RedrawRequested => {
                let Ok(surface_texture) = self.surface.get_current_texture() else {
                    return;
                };

                self.game.render(
                    &surface_texture
                        .texture
                        .create_view(&TextureViewDescriptor::default()),
                    GameContext {
                        window: &self.window,
                        device: &self.device,
                        queue: &self.queue,
                        surface_format: self.surface_config.format,
                        should_exit: None,
                    },
                );

                self.window.pre_present_notify();
                surface_texture.present();

                self.device
                    .poll(PollType::Poll)
                    .expect("failed to poll device");
            }

            WindowEvent::Resized(new_size) => {
                self.surface_config.width = new_size.width;
                self.surface_config.height = new_size.height;
                self.surface.configure(&self.device, &self.surface_config);
            }

            _ => {}
        }
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        self.handle_gilrs_events(event_loop);

        let should_exit = AtomicBool::new(false);

        self.game.update(GameContext {
            window: &self.window,
            device: &self.device,
            queue: &self.queue,
            surface_format: self.surface_config.format,
            should_exit: Some(&should_exit),
        });

        if should_exit.load(std::sync::atomic::Ordering::Relaxed) {
            self.game.end(GameContext {
                window: &self.window,
                device: &self.device,
                queue: &self.queue,
                surface_format: self.surface_config.format,
                should_exit: None,
            });

            event_loop.exit();
            return;
        }

        self.window.request_redraw();
    }

    fn handle_gilrs_events(&mut self, event_loop: &ActiveEventLoop) {
        let should_exit = AtomicBool::new(false);

        while let Some(event) = self.gilrs.next_event() {
            let ctx = GameContext {
                window: &self.window,
                device: &self.device,
                queue: &self.queue,
                surface_format: self.surface_config.format,
                should_exit: Some(&should_exit),
            };

            match event.event {
                gilrs::EventType::AxisChanged(axis, value, _) => {
                    let (positive_code, negative_code) = match axis {
                        gilrs::Axis::LeftStickX => {
                            (ButtonCode::LeftStickRight, ButtonCode::LeftStickLeft)
                        }
                        gilrs::Axis::LeftStickY => {
                            (ButtonCode::LeftStickUp, ButtonCode::LeftStickDown)
                        }
                        gilrs::Axis::RightStickX => {
                            (ButtonCode::RightStickRight, ButtonCode::RightStickLeft)
                        }
                        gilrs::Axis::RightStickY => {
                            (ButtonCode::RightStickUp, ButtonCode::RightStickDown)
                        }
                        gilrs::Axis::Unknown => (ButtonCode::Unknown, ButtonCode::Unknown),
                        gilrs::Axis::DPadX => continue,
                        gilrs::Axis::DPadY => continue,
                        gilrs::Axis::LeftZ => continue,
                        gilrs::Axis::RightZ => continue,
                    };

                    self.game.event(
                        &GameEvent::Button {
                            code: positive_code,
                            value: value.max(0.0),
                        },
                        ctx,
                    );
                    self.game.event(
                        &GameEvent::Button {
                            code: negative_code,
                            value: (-value).max(0.0),
                        },
                        ctx,
                    );
                }
                gilrs::EventType::ButtonChanged(button, value, _) => {
                    let code = match button {
                        gilrs::Button::South => ButtonCode::South,
                        gilrs::Button::East => ButtonCode::East,
                        gilrs::Button::North => ButtonCode::North,
                        gilrs::Button::West => ButtonCode::West,
                        gilrs::Button::C => ButtonCode::C,
                        gilrs::Button::Z => ButtonCode::Z,
                        gilrs::Button::LeftTrigger => ButtonCode::LeftTrigger,
                        gilrs::Button::RightTrigger => ButtonCode::RightTrigger,
                        gilrs::Button::LeftTrigger2 => ButtonCode::LeftTrigger2,
                        gilrs::Button::RightTrigger2 => ButtonCode::RightTrigger2,
                        gilrs::Button::Select => ButtonCode::Select,
                        gilrs::Button::Start => ButtonCode::Start,
                        gilrs::Button::Mode => ButtonCode::Mode,
                        gilrs::Button::LeftThumb => ButtonCode::LeftThumb,
                        gilrs::Button::RightThumb => ButtonCode::RightThumb,
                        gilrs::Button::DPadUp => ButtonCode::DPadUp,
                        gilrs::Button::DPadDown => ButtonCode::DPadDown,
                        gilrs::Button::DPadLeft => ButtonCode::DPadLeft,
                        gilrs::Button::DPadRight => ButtonCode::DPadRight,
                        gilrs::Button::Unknown => ButtonCode::Unknown,
                    };

                    self.game.event(&GameEvent::Button { code, value }, ctx);
                }
                _ => {}
            }
        }

        if should_exit.load(std::sync::atomic::Ordering::Relaxed) {
            self.game.end(GameContext {
                window: &self.window,
                device: &self.device,
                queue: &self.queue,
                surface_format: self.surface_config.format,
                should_exit: None,
            });

            event_loop.exit();
            return;
        }
    }
}
