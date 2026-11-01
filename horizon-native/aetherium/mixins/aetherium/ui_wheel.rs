// The wheel over Aetherium's screen scrolls its rows.
fn aetherium_ui_wheel(
    ui: &mut crate::options_ui::OptionsUi,
    s: &mut crate::settings::Settings,
    amount: f32,
) -> std::ops::ControlFlow<Vec<Event>> {
    crate::options_ui::aetherium::wheel(ui, s, amount)
}
