use crate::prelude::*;

pub struct TheCodeEditor {
    code_list_selection: Option<TheId>,
    grid_selection: Option<(u32, u32)>
}

impl Default for TheCodeEditor {
    fn default() -> Self {
        TheCodeEditor::new()
    }
}

impl TheCodeEditor {
    pub fn new() -> Self {
        Self {
            code_list_selection: None,
            grid_selection: None,
        }
    }

    pub fn handle_event(&mut self, event: &TheEvent, ui: &mut TheUI, ctx: &mut TheContext) -> bool {
        let mut redraw = false;

        match event {
            TheEvent::CodeEditorApply(_id) => {
                let mut atom: Option<TheAtom> = None;

                if let Some(code_list_selection) = &self.code_list_selection {
                    if let Some(widget) = ui.get_widget_id(code_list_selection.uuid) {
                        if let Some(name) = widget.value().to_string() {
                            atom = Some(self.create_atom(name.as_str()));
                        }
                    }
                }

                if let Some(atom) = atom {
                    self.set_selected_atom(ui, atom);
                    self.set_grid_selection_ui(ui, ctx);
                    redraw = true;
                }
            }
            TheEvent::CodeEditorSelectionChanged(_id, selection) => {
                self.grid_selection = *selection;
                self.set_grid_selection_ui(ui, ctx);
                redraw = true;
                /*
                ui.set_widget_disabled_state(
                    "Apply Code",
                    ctx,
                    selection.is_none() || self.code_list_selection.is_none(),
                );
                self.editor_selection = *selection;

                // Generate the Atom UI
                let mut text_layout = TheTextLayout::new(TheId::empty());
                if let Some(selection) = selection {
                    if let Some(layout) = ui.get_code_layout("Code Editor") {
                        if let Some(code_view) = layout.code_view_mut().as_code_view() {
                            let grid = code_view.code_grid();

                            if let Some(atom) = grid.code.get(selection) {
                                text_layout = atom.to_text_layout();
                            }
                        }
                    }
                }

                ui.canvas
                    .right
                    .as_mut()
                    .unwrap()
                    .center
                    .as_mut()
                    .unwrap()
                    .set_layout(text_layout);
                ctx.ui.relayout = true;
                */
            }
            TheEvent::StateChanged(id, _state) => {

                if id.name == "Code List Item" {
                    self.code_list_selection = Some(id.clone());

                    /*
                    let mut atom: Option<TheAtom> = None;

                    if let Some(widget) = ui.get_widget_id(id.uuid) {
                        if let Some(name) = widget.value().to_string() {
                            atom = Some(self.create_atom(name.as_str()));
                        }
                    }

                    if let Some(atom) = atom {
                        if let Some(grid_selection) = &self.grid_selection {
                            if let Some(layout) = ui.get_code_layout("Code Editor") {
                                if let Some(code_view) = layout.code_view_mut().as_code_view() {
                                    code_view.set_grid_atom(*grid_selection, atom);
                                    self.set_grid_selection_ui(ui, ctx);
                                    // ctx.ui.send(TheEvent::CodeEditorSelectionChanged(
                                    //     id.clone(),
                                    //     Some(*grid_selection),
                                    // ));
                                }
                            }
                        }
                    }*/
                }

                redraw = true;
            }
            TheEvent::ValueChanged(id, value) => {
                if id.name == "Atom Integer Edit" {
                    if let Some(v) = value.to_i32() {
                        self.set_selected_atom(ui, TheAtom::Value(TheValue::Int(v)));
                    }
                }
                redraw = true;
            }
            _ => {}
        }

        redraw
    }

    pub fn set_grid_selection_ui(&mut self, ui: &mut TheUI, ctx: &mut TheContext) {
        if let Some(atom) = self.get_selected_atom(ui) {
            if let Some(layout) = ui.get_hlayout("Code Top Toolbar") {
                layout.clear();
                atom.to_layout(layout);
                layout.relayout(ctx);
                ctx.ui.redraw_all = true;
            }
        } else if let Some(layout) = ui.get_hlayout("Code Top Toolbar") {
            layout.clear();
            ctx.ui.redraw_all = true;
        }
    }

    /// Returns a clone of the currently selected atom (if any).
    pub fn get_selected_atom(&mut self, ui: &mut TheUI) -> Option<TheAtom> {
        if let Some(grid_selection) = self.grid_selection {
            if let Some(layout) = ui.get_code_layout("Code Editor") {
                if let Some(code_view) = layout.code_view_mut().as_code_view() {
                    let grid = code_view.code_grid();

                    if let Some(atom) = grid.code.get(&grid_selection) {
                        return Some(atom.clone());
                    }
                }
            }
        }
        None
    }

    /// Set the atom at the current position.
    pub fn set_selected_atom(&mut self, ui: &mut TheUI, atom: TheAtom) {
        if let Some(grid_selection) = self.grid_selection {
            if let Some(layout) = ui.get_code_layout("Code Editor") {
                if let Some(code_view) = layout.code_view_mut().as_code_view() {
                    code_view.set_grid_atom(grid_selection, atom);
                }
            }
        }
    }

    /// Create an atom for the given name.
    pub fn create_atom(&self, name: &str) -> TheAtom {
        match name {
            "Integer" => TheAtom::Value(TheValue::Int(1)),
            "Add" => TheAtom::Add(),
            "Multiply" => TheAtom::Multiply(),
            _ => TheAtom::Stop,
        }
    }

    /// Builds the UI canvas
    pub fn build_canvas(&self, ctx: &mut TheContext) -> TheCanvas {
        let mut canvas: TheCanvas = TheCanvas::new();

        // Left code list

        let mut list_canvas: TheCanvas = TheCanvas::new();

        let mut code_layout = TheListLayout::new(TheId::named("Code List"));
        code_layout.limiter_mut().set_max_width(150);

        let mut item = TheListItem::new(TheId::named("Code List Item"));
        item.set_text("Integer".to_string());
        item.set_associated_layout(code_layout.id().clone());
        code_layout.add_item(item, ctx);

        let mut item = TheListItem::new(TheId::named("Code List Item"));
        item.set_text("Add".to_string());
        item.set_associated_layout(code_layout.id().clone());
        code_layout.add_item(item, ctx);

        let mut item = TheListItem::new(TheId::named("Code List Item"));
        item.set_text("Multiply".to_string());
        item.set_associated_layout(code_layout.id().clone());
        code_layout.add_item(item, ctx);

        code_layout.select_first_item(ctx);
        list_canvas.set_layout(code_layout);

        let mut list_toolbar_canvas = TheCanvas::new();
        let mut toolbar_hlayout = TheHLayout::new(TheId::empty());
        toolbar_hlayout.set_background_color(None);
        toolbar_hlayout.set_margin(vec4i(5, 2, 5, 2));
        list_toolbar_canvas.set_layout(toolbar_hlayout);
        list_toolbar_canvas.set_widget(TheTraybar::new(TheId::empty()));
        list_canvas.set_top(list_toolbar_canvas);

        // Top Toolbar
        let mut top_toolbar_canvas = TheCanvas::new();
        let mut compile_button = TheTraybarButton::new(TheId::named("Compile"));
        //compile_button.set_disabled(true);
        compile_button.set_text("Compile".to_string());
        let mut toolbar_hlayout = TheHLayout::new(TheId::named("Code Top Toolbar"));
        toolbar_hlayout.set_background_color(None);
        toolbar_hlayout.set_margin(vec4i(5, 2, 5, 2));
        //toolbar_hlayout.limiter_mut().set_max_height(27);
        top_toolbar_canvas.set_layout(toolbar_hlayout);
        top_toolbar_canvas.set_widget(TheTraybar::new(TheId::empty()));

        // Bottom Toolbar
        let mut bottom_toolbar_canvas = TheCanvas::new();

        let mut compile_button = TheTraybarButton::new(TheId::named("Compile"));
        //compile_button.set_disabled(true);
        compile_button.set_text("Compile".to_string());

        let mut toolbar_hlayout = TheHLayout::new(TheId::empty());
        toolbar_hlayout.set_background_color(None);
        toolbar_hlayout.set_margin(vec4i(5, 2, 5, 2));
        toolbar_hlayout.add_widget(Box::new(compile_button));
        toolbar_hlayout.limiter_mut().set_max_height(27);

        bottom_toolbar_canvas.set_layout(toolbar_hlayout);
        bottom_toolbar_canvas.set_widget(TheTraybar::new(TheId::empty()));

        // ---

        let code_layout = TheCodeLayout::new(TheId::named("Code Editor"));

        canvas.set_layout(code_layout);
        canvas.set_left(list_canvas);
        canvas.set_top(top_toolbar_canvas);
        canvas.set_bottom(bottom_toolbar_canvas);

        canvas
    }

}