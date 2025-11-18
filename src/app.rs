use std::time::Duration;

use lib_app::{AppContext, AppEvent, AppFlow, AppHandler};
use lib_gpu::TextureView;
use lib_input::Mapper;
use lib_math::{f32::FVec2, vec2, vec2s, vec4s};
use lib_renderer::{Camera, Quad, QuadBuffer, Renderer, Sprite};

use crate::input::{Input, InputBindings};

#[derive(Debug)]
pub struct Game {
    renderer: Renderer,
    quads: QuadBuffer,
    mapper: Mapper<Input>,
    pos: FVec2,
}

impl AppHandler for Game {
    const TITLE: &str = "Drill";

    fn new(ctx: AppContext<'_>) -> Self {
        Self {
            renderer: Renderer::new(ctx.into()),
            quads: QuadBuffer::new_init(
                &[Quad {
                    center: vec2s!(0.0),
                    layer: 0.0,
                    sprite: Sprite {
                        center: vec2s!(1.0 / 40.0),
                        extents: vec2s!(1.0 / 40.0),
                    },
                }],
                ctx.into(),
            ),
            mapper: Mapper::new(&InputBindings::default()),
            pos: FVec2::ZERO,
        }
    }

    fn update(&mut self, delta_time: Duration, ctx: AppContext<'_>) -> AppFlow {
        let input = self.mapper.map();

        self.pos += vec2!(input.x.value(), input.y.value()) * 10.0 * delta_time.as_secs_f32();

        self.quads.index(0).write(
            &Quad {
                center: self.pos.as_nonsimd(),
                sprite: Sprite {
                    center: vec2s!(1.0 / 40.0),
                    extents: vec2s!(1.0 / 40.0),
                },
                layer: 0.0,
            },
            ctx.into(),
        );

        AppFlow::Continue
    }

    fn event(&mut self, event: AppEvent<'_>, _ctx: AppContext<'_>) -> AppFlow {
        match event {
            AppEvent::Device { device: _, event } => self.mapper.event(event),
            AppEvent::CloseRequested => return AppFlow::Exit,
            _ => {}
        }

        AppFlow::Continue
    }

    fn draw(&mut self, output: &TextureView, ctx: AppContext<'_>) {
        self.renderer.render(
            self.quads.slice(..),
            &Camera {
                center: vec2s!(0.0),
                clear_color: vec4s!(1.0, 0.0, 0.0, 1.0),
                ortho_size: 8.0,
            },
            output,
            ctx.into(),
        );
    }
}
