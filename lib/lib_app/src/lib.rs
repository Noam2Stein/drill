use std::time::Duration;

use lib_gpu::{
    Color, CommandEncoderDescriptor, Device, LoadOp, Operations, Queue, RenderPassColorAttachment,
    RenderPassDescriptor, StoreOp, TextureFormat, TextureView,
};
use lib_window::{DeviceEvent, DeviceId, Window, WindowAttributes};

mod runner;

pub trait AppHandler {
    const TITLE: &str = "Untitled App";

    fn window_attributes() -> WindowAttributes {
        WindowAttributes::default().with_title(Self::TITLE)
    }

    fn new(_ctx: AppContext<'_>) -> Self;

    fn update(&mut self, _delta_time: Duration, _ctx: AppContext<'_>) -> AppFlow {
        AppFlow::Continue
    }

    fn event(&mut self, event: AppEvent<'_>, _ctx: AppContext<'_>) -> AppFlow {
        match event {
            AppEvent::CloseRequested => AppFlow::Exit,
            _ => AppFlow::Continue,
        }
    }

    fn draw(&mut self, output: &TextureView, ctx: AppContext<'_>) {
        let mut encoder = ctx
            .device
            .create_command_encoder(&CommandEncoderDescriptor::default());

        encoder.begin_render_pass(&RenderPassDescriptor {
            label: None,
            color_attachments: &[Some(RenderPassColorAttachment {
                view: output,
                ops: Operations {
                    load: LoadOp::Clear(Color::BLACK),
                    store: StoreOp::Store,
                },
                depth_slice: None,
                resolve_target: None,
            })],
            depth_stencil_attachment: None,
            occlusion_query_set: None,
            timestamp_writes: None,
        });

        ctx.queue.submit([encoder.finish()]);
    }
}

#[derive(Debug, Clone, Copy)]
pub struct AppContext<'a> {
    pub window: &'a Window,
    pub device: &'a Device,
    pub queue: &'a Queue,
    pub surface_format: TextureFormat,
}

#[derive(Debug, Clone, Copy)]
pub enum AppFlow {
    Continue,
    Exit,
}

#[non_exhaustive]
#[derive(Debug, Clone, Copy)]
pub enum AppEvent<'a> {
    CloseRequested,
    Device {
        device: DeviceId,
        event: DeviceEvent<'a>,
    },
    UnhandledWindowEvent(&'a lib_window::event::WindowEvent),
    UnhandledDeviceEvent {
        device: lib_window::event::DeviceId,
        event: &'a lib_window::event::DeviceEvent,
    },
}

#[macro_export]
macro_rules! app_main {
    ($Game:ty) => {
        fn main() {
            $crate::hidden::run_game::<$Game>();
        }
    };
}

#[doc(hidden)]
pub mod hidden {
    pub use crate::runner::run_game;
}
