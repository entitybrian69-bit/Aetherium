// The info overlay and the waypoint markers.
fn aetherium_hud(
    game: &mut crate::game::Game<'_>,
    batch: &mut horizon_client::hud::HudBatch,
    corners: &mut horizon_client::extras_hud::Corners,
) {
    crate::game::aetherium_rt::hud(game, batch, corners);
}
