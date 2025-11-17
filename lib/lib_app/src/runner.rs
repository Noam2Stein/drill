use std::{sync::Arc, time::Instant};

use gilrs::Gilrs;
use lib_gpu::{
    Device, DeviceDescriptor, Instance, PollType, Queue, RequestAdapterOptions, Surface,
    SurfaceConfiguration, TextureViewDescriptor,
};
use lib_window::{
    ButtonCode, ButtonEvent, DeviceEvent,
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, EventLoop},
    keyboard::{KeyCode, PhysicalKey},
    window::{Fullscreen, Window},
};

use crate::{AppContext, AppEvent, AppFlow, AppHandler, DeviceId};

pub fn run_game<T: AppHandler>() {
    let event_loop = EventLoop::new().expect("Failed to create event loop");

    let mut application = GameRunner::<T>::Uninitialized;

    event_loop
        .run_app(&mut application)
        .expect("Failed to run game");
}

enum GameRunner<T: AppHandler> {
    Uninitialized,
    Initialized(InitializedGameRunner<T>),
}

struct InitializedGameRunner<T: AppHandler> {
    window: Arc<Window>,
    device: Device,
    queue: Queue,
    surface: Surface<'static>,
    surface_config: SurfaceConfiguration,
    gilrs: Gilrs,
    alt_left_is_held: bool,
    alt_right_is_held: bool,
    game: T,
    last_instant: Instant,
}

impl<T: AppHandler> ApplicationHandler for GameRunner<T> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let GameRunner::Uninitialized = self else {
            return;
        };

        *self = GameRunner::Initialized(InitializedGameRunner::new(event_loop));
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        let GameRunner::Initialized(init_self) = self else {
            return;
        };

        init_self.poll_gilrs_events(event_loop);

        let now = Instant::now();
        let delta_time = now.duration_since(init_self.last_instant);
        init_self.last_instant = now;

        handle_gameflow!(
            event_loop,
            init_self.game.update(
                delta_time,
                AppContext {
                    window: &init_self.window,
                    device: &init_self.device,
                    queue: &init_self.queue,
                    surface_format: init_self.surface_config.format,
                },
            )
        );

        init_self.window.request_redraw();
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: lib_window::window::WindowId,
        event: WindowEvent,
    ) {
        let Self::Initialized(init_self) = self else {
            return;
        };

        init_self.fsswitch_window_event(&event);

        let game_event = match &event {
            WindowEvent::CloseRequested => AppEvent::CloseRequested,
            WindowEvent::KeyboardInput {
                device_id,
                event,
                is_synthetic: _,
            } => AppEvent::Device {
                device: DeviceId::Winit(*device_id),
                event: DeviceEvent::Key(event),
            },
            _ => AppEvent::UnhandledWindowEvent(&event),
        };

        handle_gameflow!(
            event_loop,
            init_self.game.event(
                game_event,
                AppContext {
                    window: &init_self.window,
                    device: &init_self.device,
                    queue: &init_self.queue,
                    surface_format: init_self.surface_config.format,
                },
            )
        );

        match &event {
            WindowEvent::RedrawRequested => 'redraw: {
                let Ok(texture) = init_self.surface.get_current_texture() else {
                    break 'redraw;
                };

                init_self.game.draw(
                    &texture
                        .texture
                        .create_view(&TextureViewDescriptor::default()),
                    AppContext {
                        window: &init_self.window,
                        device: &init_self.device,
                        queue: &init_self.queue,
                        surface_format: init_self.surface_config.format,
                    },
                );

                init_self.window.pre_present_notify();
                texture.present();

                init_self
                    .device
                    .poll(PollType::Poll)
                    .expect("Failed to poll");
            }
            WindowEvent::Resized(size) => {
                init_self.surface_config.width = size.width;
                init_self.surface_config.height = size.height;

                init_self
                    .surface
                    .configure(&init_self.device, &init_self.surface_config);
            }
            _ => {}
        }
    }

    fn device_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        device_id: lib_window::event::DeviceId,
        event: lib_window::event::DeviceEvent,
    ) {
        let GameRunner::Initialized(init_self) = self else {
            return;
        };

        let game_event: AppEvent<'_> = match event {
            _ => AppEvent::UnhandledDeviceEvent {
                device: device_id,
                event: &event,
            },
        };

        handle_gameflow!(
            event_loop,
            init_self.game.event(
                game_event,
                AppContext {
                    window: &init_self.window,
                    device: &init_self.device,
                    queue: &init_self.queue,
                    surface_format: init_self.surface_config.format,
                },
            )
        );
    }
}

impl<T: AppHandler> InitializedGameRunner<T> {
    fn new(event_loop: &ActiveEventLoop) -> Self {
        let window = Arc::new(
            event_loop
                .create_window(T::window_attributes())
                .expect("Failed to create main window"),
        );

        let instance = Instance::default();

        let adapter =
            pollster::block_on(instance.request_adapter(&RequestAdapterOptions::default()))
                .expect("Failed to get adapter");

        let (device, queue) =
            pollster::block_on(adapter.request_device(&DeviceDescriptor::default()))
                .expect("Failed to get device");

        let surface = instance
            .create_surface(window.clone())
            .expect("Failed to create surface");

        let surface_config = surface
            .get_default_config(
                &adapter,
                window.inner_size().width,
                window.inner_size().height,
            )
            .expect("Failed to get default surface configuration");

        surface.configure(&device, &surface_config);

        let gilrs = Gilrs::new().expect("Failed to initialize gilrs (gamepad tool)");

        let game = T::new(AppContext {
            window: &window,
            device: &device,
            queue: &queue,
            surface_format: surface_config.format,
        });

        let last_instant = Instant::now();

        Self {
            window,
            device,
            queue,
            surface,
            surface_config,
            alt_left_is_held: false,
            alt_right_is_held: false,
            gilrs,
            game,
            last_instant,
        }
    }

    fn fsswitch_window_event(&mut self, event: &WindowEvent) {
        match event {
            WindowEvent::KeyboardInput {
                device_id: _,
                event,
                is_synthetic: _,
            } => {
                if event.repeat {
                    return;
                }

                let PhysicalKey::Code(keycode) = event.physical_key else {
                    return;
                };

                match keycode {
                    KeyCode::AltLeft => self.alt_left_is_held = event.state.is_pressed(),
                    KeyCode::AltRight => self.alt_left_is_held = event.state.is_pressed(),

                    KeyCode::Enter if event.state.is_pressed() => {
                        if self.alt_left_is_held || self.alt_right_is_held {
                            match self.window.fullscreen() {
                                Some(_) => self.window.set_fullscreen(None),
                                None => self
                                    .window
                                    .set_fullscreen(Some(Fullscreen::Borderless(None))),
                            }
                        }
                    }

                    _ => {}
                }
            }
            _ => {}
        }
    }

    fn poll_gilrs_events(&mut self, event_loop: &ActiveEventLoop) {
        while let Some(event) = self.gilrs.next_event() {
            let device = DeviceId::Gilrs(event.id);

            let ctx = AppContext {
                window: &self.window,
                device: &self.device,
                queue: &self.queue,
                surface_format: self.surface_config.format,
            };

            match event.event {
                gilrs::EventType::Connected => handle_gameflow!(
                    event_loop,
                    self.game.event(
                        AppEvent::Device {
                            device,
                            event: DeviceEvent::Connected
                        },
                        ctx,
                    )
                ),
                gilrs::EventType::Disconnected => handle_gameflow!(
                    event_loop,
                    self.game.event(
                        AppEvent::Device {
                            device,
                            event: DeviceEvent::Disconnected
                        },
                        ctx,
                    )
                ),
                gilrs::EventType::AxisChanged(axis, value, _) => {
                    let (positive_button, negative_button) = match axis {
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

                    handle_gameflow!(
                        event_loop,
                        self.game.event(
                            AppEvent::Device {
                                device,
                                event: DeviceEvent::Button(&ButtonEvent {
                                    button: positive_button,
                                    value: value.max(0.0),
                                })
                            },
                            ctx
                        )
                    );

                    handle_gameflow!(
                        event_loop,
                        self.game.event(
                            AppEvent::Device {
                                device,
                                event: DeviceEvent::Button(&ButtonEvent {
                                    button: negative_button,
                                    value: (-value).max(0.0),
                                }),
                            },
                            ctx
                        )
                    );
                }
                gilrs::EventType::ButtonChanged(button, value, _) => {
                    let button = match button {
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

                    handle_gameflow!(
                        event_loop,
                        self.game.event(
                            AppEvent::Device {
                                device,
                                event: DeviceEvent::Button(&ButtonEvent { button, value }),
                            },
                            ctx
                        )
                    );
                }
                gilrs::EventType::ForceFeedbackEffectCompleted => {}
                gilrs::EventType::Dropped => {}
                _ => {}
            }
        }
    }
}

macro_rules! handle_gameflow {
    ($event_loop:expr, $flow:expr) => {
        match $flow {
            AppFlow::Continue => {}
            AppFlow::Exit => {
                $event_loop.exit();
                return;
            }
        }
    };
}

use handle_gameflow;
