// Aetherium's zoom, unless another mod's came first.
fn aetherium_zoom(
    game: &crate::game::Game<'_>,
    zoom: &mut Option<crate::game::native_hooks::NativeZoom>,
) {
    crate::game::aetherium_rt::zoom(game, zoom);
}
