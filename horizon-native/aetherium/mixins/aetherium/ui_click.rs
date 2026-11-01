// A press or drag on Aetherium's screen.
// (Local to the target body in options_ui.rs: `Event`/`Row` are the module's own, written as the target writes them.)
fn aetherium_ui_click(
    ui: &mut crate::options_ui::OptionsUi,
    s: &mut crate::settings::Settings,
    x: f32,
    y: f32,
    drag: bool,
    right: bool,
) -> std::ops::ControlFlow<Vec<Event>> {
    crate::options_ui::aetherium::click(ui, s, x, y, drag, right)
}
