// Aetherium's keys: zoom, zoom in, zoom out, add waypoint, open Aetherium.
fn aetherium_key(game: &mut crate::game::Game<'_>, key: &str, pressed: bool) {
    crate::game::aetherium_rt::key(game, key, pressed);
}
