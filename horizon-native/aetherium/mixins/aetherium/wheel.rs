// The wheel zooms while zoomed.
fn aetherium_wheel(game: &crate::game::Game<'_>, steps: f32, claimed: &mut bool) {
    crate::game::aetherium_rt::wheel(game, steps, claimed);
}
