use crate::prelude::*;

/// All events which are handled by the framework
#[derive(Clone, Debug)]
pub enum TheEvent {
    // These events are passed to the on_event function of the widgets and cover user interaction.
    MouseDown(TheValue),
    Hover(TheValue),
    MouseDragged(TheValue),
    MouseUp(TheValue),

    // These events define widget states.
    StateChanged(TheWidgetId, TheWidgetState),
    SetState(String, TheWidgetState),

    ValueChanged(TheWidgetId, TheValue),
    SetValue(String, TheValue),

    GainedFocus(TheWidgetId),
    LostFocus(TheWidgetId),
    GainedHover(TheWidgetId),
    LostHover(TheWidgetId),
}