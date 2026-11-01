// Aetherium's page in Options: Aetherium's own screen instead of the stock list.
fn aetherium_ui_draw(
    ui: &mut crate::options_ui::OptionsUi,
    s: &crate::settings::Settings,
    batch: &mut crate::hud::HudBatch,
    atlas: &crate::hud::HudAtlas,
    cursor: (f32, f32),
) -> std::ops::ControlFlow<()> {
    crate::options_ui::aetherium::draw(ui, s, batch, atlas, cursor)
}
