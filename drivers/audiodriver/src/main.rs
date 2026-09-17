use audiodriver as ad;

/// Punkt wejścia binarki pomocniczej.
///
/// Właściwy sterownik działa w driver space przez `ds_entry` (patrz
/// `driverspace/src/main.rs`); ta binarka jest cienkim wrapperem CLI, który
/// pozwala sprawdzić stan gniazda jack z poziomu hosta.
fn main() {
    let mut mgr = ad::jacklib::JackMgr::new();
    mgr.tick();

    println!(
        "audiodriver: jack present = {}, amp on = {}",
        mgr.present(),
        mgr.amp_enabled()
    );
}