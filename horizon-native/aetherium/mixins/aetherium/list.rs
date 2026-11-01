// Aetherium's row in Options > Mods, with its keys.
fn aetherium_listed(mods: &mut Vec<crate::game::native_hooks::NativeMod>) {
    crate::game::aetherium_rt::listed(mods);
}
