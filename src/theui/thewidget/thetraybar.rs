use crate::prelude::*;

pub struct TheTraybar {
    id: TheId,

    limiter: TheSizeLimiter,

    dim: TheDim,
    is_dirty: bool,
}

impl TheWidget for TheTraybar {
    fn new(id: TheId) -> Self
    where
        Self: Sized,
    {
        let mut limiter = TheSizeLimiter::new();
        limiter.set_max_height(27);

        Self {
            id,
            limiter,

            dim: TheDim::zero(),
            is_dirty: false,
        }
    }

    fn id(&self) -> &TheId {
        &self.id
    }

    // fn on_event(&mut self, event: &TheEvent, ctx: &mut TheContext) -> bool {
    //     false
    // }

    fn dim(&self) -> &TheDim {
        &self.dim
    }

    fn dim_mut(&mut self) -> &mut TheDim {
        &mut self.dim
    }

    fn set_dim(&mut self, dim: TheDim) {
        if self.dim != dim {
            self.dim = dim;
            self.is_dirty = true;
        }
    }

    fn limiter(&self) -> &TheSizeLimiter {
        &self.limiter
    }

    fn limiter_mut(&mut self) -> &mut TheSizeLimiter {
        &mut self.limiter
    }

    fn needs_redraw(&mut self) -> bool {
        self.is_dirty
    }

    fn draw(
        &mut self,
        buffer: &mut TheRGBABuffer,
        style: &mut Box<dyn TheStyle>,
        ctx: &mut TheContext,
    ) {
        if !self.dim().is_valid() {
            return;
        }

        let stride = buffer.stride();
        let utuple = &self.dim.to_buffer_utuple();

        ctx.draw.rect(
            buffer.pixels_mut(),
            utuple,
            stride,
            style.theme().color(TraybarBorder),
        );

        ctx.draw.rect(
            buffer.pixels_mut(),
            &(utuple.0 + 1, utuple.1 + 1, utuple.2 - 1, utuple.3 - 1),
            stride,
            style.theme().color(TraybarBackground),
        );

        ctx.draw.rect(
            buffer.pixels_mut(),
            &(utuple.0 + 1, utuple.1 + utuple.3 - 1, utuple.2 - 1, 1),
            stride,
            style.theme().color(TraybarBottomBorder),
        );

        self.is_dirty = false;
    }
}