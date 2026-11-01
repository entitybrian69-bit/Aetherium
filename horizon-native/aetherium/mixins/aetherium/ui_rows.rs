// Video Settings' rows with "Aetherium Settings..." under Horizon's own buttons.
fn aetherium_ui_rows(
    ui: &crate::options_ui::OptionsUi,
    s: &crate::settings::Settings,
) -> std::ops::ControlFlow<Vec<Row>> {
    crate::options_ui::aetherium::video_rows(ui, s)
}
