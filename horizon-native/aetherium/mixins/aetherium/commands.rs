// The commands chat completes.
fn aetherium_commands(commands: &mut Vec<&'static str>) {
    crate::game::aetherium_rt::commands(commands);
}
