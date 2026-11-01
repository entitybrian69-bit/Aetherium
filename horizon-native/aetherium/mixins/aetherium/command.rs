// /aetherium opens the screen; /wp manages waypoints.
fn aetherium_command(game: &mut crate::game::Game<'_>, command: &str, ran: &mut bool) {
    crate::game::aetherium_rt::command(game, command, ran);
}
