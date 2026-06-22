//! Shared, deterministic game logic that BOTH the client (for prediction) and the
//! server (as the authority) run, so the two stay in agreement. Keep simulation
//! rules here; keep rendering in the client and networking/authority in the server.

/// Advances one tick of the simulation. Pure and deterministic so client and
/// server compute the same result from the same input.
pub fn step(tick: u64) -> u64 {
    tick.wrapping_add(1)
}

/// A friendly label, shared so both sides agree on the game's identity.
pub fn name() -> &'static str {
    "hello multiplayer"
}
