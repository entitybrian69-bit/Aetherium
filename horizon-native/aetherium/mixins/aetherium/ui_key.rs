// Keys on Aetherium's screen: Escape is Done, Enter is Apply, the arrows change tabs.
fn aetherium_ui_key(
    ui: &mut crate::options_ui::OptionsUi,
    s: &mut crate::settings::Settings,
    key: winit::keyboard::KeyCode,
) -> std::ops::ControlFlow<Vec<crate::options_ui::Event>> {
    crate::options_ui::aetherium::key(ui, s, key)
}
