use crate::prelude::*;

pub struct TheListLayout {
    id: TheId,
    limiter: TheSizeLimiter,

    dim: TheDim,

    widgets: Vec<Box<dyn TheWidget>>,

    list_buffer: TheRGBABuffer,

    vertical_scrollbar: Box<dyn TheWidget>,
    vertical_scrollbar_visible: bool,

    margin: Vec4i,

    background: Option<TheThemeColors>,
}

impl TheLayout for TheListLayout {
    fn new(name: String) -> Self
    where
        Self: Sized,
    {
        Self {
            id: TheId::new(name),
            limiter: TheSizeLimiter::new(),

            dim: TheDim::zero(),

            widgets: vec![],
            list_buffer: TheRGBABuffer::empty(),

            vertical_scrollbar: Box::new(TheVerticalScrollbar::new(
                "Vertical Scrollbar".to_string(),
            )),
            vertical_scrollbar_visible: false,

            margin: vec4i(0, 0, 0, 0),

            background: Some(TextLayoutBackground),
        }
    }

    fn id(&self) -> &TheId {
        &self.id
    }

    fn set_margin(&mut self, margin: Vec4i) {
        self.margin = margin;
    }

    fn set_background_color(&mut self, color: Option<TheThemeColors>) {
        self.background = color;
    }

    fn widgets(&mut self) -> &mut Vec<Box<dyn TheWidget>> {
        &mut self.widgets
    }

    fn get_widget_at_coord(&mut self, coord: Vec2i) -> Option<&mut Box<dyn TheWidget>> {
        if !self.dim.contains(coord) {
            return None;
        }

        if self.vertical_scrollbar_visible && self.vertical_scrollbar.dim().contains(coord) {
            return Some(&mut self.vertical_scrollbar);
        }

        let mut scroll_offset = vec2i(0, 0);
        if let Some(scroll_bar) = self.vertical_scrollbar.as_vertical_scrollbar() {
            scroll_offset = vec2i(0, scroll_bar.scroll_offset());
        }

        let widgets = self.widgets();
        widgets.iter_mut().find(|w| w.dim().contains(coord + scroll_offset))
    }

    fn get_widget(
        &mut self,
        name: Option<&String>,
        uuid: Option<&Uuid>,
    ) -> Option<&mut Box<dyn TheWidget>> {
        if self.vertical_scrollbar_visible && self.vertical_scrollbar.id().matches(name, uuid) {
            return Some(&mut self.vertical_scrollbar);
        }

        self.widgets.iter_mut().find(|w| w.id().matches(name, uuid))
    }

    fn needs_redraw(&mut self) -> bool {
        if self.vertical_scrollbar_visible && self.vertical_scrollbar.needs_redraw() {
            return true;
        }

        for i in 0..self.widgets.len() {
            if self.widgets[i].needs_redraw() {
                return true;
            }
        }
        false
    }

    fn dim(&self) -> &TheDim {
        &self.dim
    }

    fn dim_mut(&mut self) -> &mut TheDim {
        &mut self.dim
    }

    fn set_dim(&mut self, dim: TheDim, _ctx: &mut TheContext) {
        if self.dim != dim {
            self.dim = dim;

            let x = 1;
            let mut y = 1;
            let mut width = dim.width;

            let items = self.widgets.len() as i32;
            let mut total_height = 1 + items * 17 + 1;
            if items > 0 {
                total_height += (items - 1) * 3;
            }

            self.vertical_scrollbar
                .set_dim(TheDim::new(dim.x + width - 13, dim.y, 13, dim.height));
            self.vertical_scrollbar
                .dim_mut()
                .set_buffer_offset(self.dim.buffer_x + width - 13, self.dim.buffer_y);

            if let Some(scroll_bar) = self.vertical_scrollbar.as_vertical_scrollbar() {
                scroll_bar.set_total_height(total_height);
                self.vertical_scrollbar_visible = scroll_bar.needs_scrollbar();
            }

            if self.vertical_scrollbar_visible {
                width -= 13;
            }

            self.list_buffer.set_dim(TheDim::new(0, 0, width, total_height));

            for index in 0..items {
                let i = index as usize;

                self.widgets[i].set_dim(TheDim::new(dim.x + x, dim.y + y, width - 1, 17));
                self.widgets[i]
                    .dim_mut()
                    .set_buffer_offset(x, y);

                y += 17 + 3;
            }
        }
    }

    fn limiter(&self) -> &TheSizeLimiter {
        &self.limiter
    }

    fn limiter_mut(&mut self) -> &mut TheSizeLimiter {
        &mut self.limiter
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

        let stride = self.list_buffer.stride();
        let utuple: (usize, usize, usize, usize) = self.list_buffer.dim().to_buffer_utuple();

        ctx.draw.rect(
            self.list_buffer.pixels_mut(),
            &utuple,
            stride,
            style.theme().color(ListLayoutBackground),
        );

        if self.vertical_scrollbar_visible {
            self.vertical_scrollbar.draw(buffer, style, ctx);
        }

        let items = self.widgets.len();

        for i in 0..items {
            self.widgets[i].draw(&mut self.list_buffer, style, ctx);
        }

        if let Some(scroll_bar) = self.vertical_scrollbar.as_vertical_scrollbar() {
            let offset = scroll_bar.scroll_offset();
            let range = offset..offset + self.dim.height;
            buffer.copy_vertical_range_into(self.dim.buffer_x, self.dim.buffer_y, &self.list_buffer, range);
        }

    }

    /// Convert to the list layout trait
    fn as_list_layout(&mut self) -> Option<&mut dyn TheListLayoutTrait> {
        Some(self)
    }
}

/// TheListLayout specific functions.
pub trait TheListLayoutTrait {
    /// Add an item
    fn add_item(&mut self, item: TheListItem);
    /// A new item was selected, manage the selection states
    fn new_item_selected(&mut self, item: TheId);
}

impl TheListLayoutTrait for TheListLayout {
    fn add_item(&mut self, mut item: TheListItem) {
        item.set_associated_layout(self.id().clone());
        self.widgets.push(Box::new(item));
    }
    fn new_item_selected(&mut self, item: TheId) {
        for w in &mut self.widgets {
            if !w.id().equals(&Some(item.clone())) {
                w.set_state(TheWidgetState::None);
            }
        }
    }
}