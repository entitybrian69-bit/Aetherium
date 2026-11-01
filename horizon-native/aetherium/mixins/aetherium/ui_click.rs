// A press or drag on Aetherium's screen.
fn aetherium_ui_click(
    ui: &mut crate::options_ui::OptionsUi,
    s: &mut crate::settings::Settings,
    x: f32,
    y: f32,
    drag: bool,
    right: bool,
) -> std::ops::ControlFlow<Vec<crate::options_ui::Event>> {
    crate::options_ui::aetherium::click(ui, s, x, y, drag, right)
}
