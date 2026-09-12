use std::sync::Mutex;

use dtt_application::CancellationToken;

#[derive(Default)]
pub struct DesktopState {
    pub active_generation: Mutex<Option<ActiveGeneration>>,
}

pub struct ActiveGeneration {
    pub id: u64,
    pub cancellation_token: CancellationToken,
}
