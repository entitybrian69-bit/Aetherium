// Every frame: Aetherium's settings, waypoints and zoom.
fn aetherium_frame(game: &mut crate::game::Game<'_>, elapsed: std::time::Duration) {
    crate::game::aetherium_rt::frame(game, elapsed);
}
