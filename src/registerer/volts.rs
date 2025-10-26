use std::sync::{atomic::AtomicU64, Arc, Mutex};

use prometheus_client::{
    metrics::{family::Family, gauge::Gauge},
    registry::Registry,
};

use crate::{
    metrics::{
        volts::{VoltKind, VoltLabels},
        Registerer,
    },
    parser::volts::VoltsState,
};

#[derive(Debug)]
pub struct VoltsRegisterer {
    pub registry: Arc<Mutex<Registry>>,
}

impl Registerer for VoltsRegisterer {
    type Item = VoltsState;

    async fn register(&self, state: Self::Item) -> anyhow::Result<()> {
        let volts_family = Family::<VoltLabels, Gauge<f64, AtomicU64>>::default();
        {
            let mut registry = self.registry.lock().expect("failed to lock registry mutex");
            registry.register(
                "raspi_volts",
                "Current voltage",
                volts_family.clone(),
            );
        }

        volts_family.get_or_create(&VoltLabels { kind: VoltKind::Core }).set(state.core);
        volts_family.get_or_create(&VoltLabels { kind: VoltKind::SdramC }).set(state.sdram_c);
        volts_family.get_or_create(&VoltLabels { kind: VoltKind::SdramI }).set(state.sdram_i);
        volts_family.get_or_create(&VoltLabels { kind: VoltKind::SdramP }).set(state.sdram_p);

        Ok(())
    }
}
