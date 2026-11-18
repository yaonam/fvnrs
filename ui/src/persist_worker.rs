use std::sync::Arc;

use engine_core::{
    EngineCore,
    io::{persistent_path, write_file},
};
use parking_lot::RwLock;
use state::persistent::Persistent;

pub(crate) struct PersistWorker {
    engine_core: Arc<RwLock<EngineCore>>,

    persistent_snapshot: Persistent,

    persist_worker_rx: std::sync::mpsc::Receiver<()>,
}

impl PersistWorker {
    pub(crate) fn new(
        engine_core: Arc<RwLock<EngineCore>>,
        persist_worker_rx: std::sync::mpsc::Receiver<()>,
    ) -> Self {
        let persistent_snapshot = engine_core.read().persistent.clone();
        Self {
            engine_core,
            persistent_snapshot,
            persist_worker_rx,
        }
    }

    pub(crate) fn start(mut self) {
        std::thread::spawn(move || {
            while let Ok(()) = self.persist_worker_rx.recv() {
                let persistent = {
                    let persistent = &self.engine_core.read().persistent;
                    if *persistent == self.persistent_snapshot {
                        continue;
                    }
                    *persistent
                };

                if let Err(e) = write_file(persistent_path(), &persistent) {
                    log::error!("Failed to write persistent snapshot: {}", e);
                    continue;
                }

                self.persistent_snapshot = persistent;

                // Wait for 1s
                std::thread::sleep(std::time::Duration::from_secs(1));
            }
            log::error!("Persist worker exited!");
        });
    }
}
