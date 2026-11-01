// The mouse button went up over Aetherium's screen.
fn aetherium_ui_release(
    ui: &mut crate::options_ui::OptionsUi,
) -> std::ops::ControlFlow<Vec<Event>> {
    crate::options_ui::aetherium::release(ui)
}
