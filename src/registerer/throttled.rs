use std::sync::{Arc, Mutex};

use prometheus_client::{
    metrics::{family::Family, gauge::Gauge},
    registry::Registry,
};

use crate::{metrics::{throttled::{ThrottlingActiveLabels, ThrottlingKind, ThrottlingOccurredLabels}, Registerer}, parser::throttled::ThrottledState};

#[derive(Debug)]
pub struct ThrottledRegisterer {
    pub registry: Arc<Mutex<Registry>>,
}

impl Registerer for ThrottledRegisterer {
    type Item = ThrottledState;

    async fn register(&self, state: Self::Item) -> anyhow::Result<()> {
        // Substitutes Gauge for StateSet of OpenMetrics because prometheus_client doens't implement it
        let throttling_active_family = Family::<ThrottlingActiveLabels, Gauge>::default();
        // Substitutes Gauge for StateSet of OpenMetrics because prometheus_client doens't implement it
        let throttling_occurred_family = Family::<ThrottlingOccurredLabels, Gauge>::default();
        {
            let mut registry = self.registry.lock().expect("failed to lock registry mutex");
            registry.register(
                "raspi_throttling_active",
                "State about throttling active currently",
                throttling_active_family.clone(),
            );
            registry.register(
                "raspi_throttling_occurred",
                "State about throttling occurred in the past",
                throttling_occurred_family.clone(),
            );
        }

        throttling_active_family.get_or_create(&ThrottlingActiveLabels { kind: ThrottlingKind::Undervoltage }).set(state.undervoltage_detected.into());
        throttling_active_family.get_or_create(&ThrottlingActiveLabels { kind: ThrottlingKind::ArmFrequency }).set(state.arm_frequency_capped.into());
        throttling_active_family.get_or_create(&ThrottlingActiveLabels { kind: ThrottlingKind::Throttled }).set(state.currently_throttled.into());
        throttling_active_family.get_or_create(&ThrottlingActiveLabels { kind: ThrottlingKind::SoftTemperatureLimit }).set(state.soft_temperature_limit_active.into());

        {
            let metric = throttling_occurred_family.get_or_create(&ThrottlingOccurredLabels { kind: ThrottlingKind::Undervoltage });
            if state.undervoltage_has_occurred && metric.get() == 0 {
                metric.inc();
            }
        }

        {
            let metric = throttling_occurred_family.get_or_create(&ThrottlingOccurredLabels { kind: ThrottlingKind::ArmFrequency });
            if state.arm_frequency_capping_has_occurred && metric.get() == 0 {
                metric.inc();
            }
        }

        {
            let metric = throttling_occurred_family.get_or_create(&ThrottlingOccurredLabels { kind: ThrottlingKind::Throttled });
            if state.throttling_has_occurred && metric.get() == 0 {
                metric.inc();
            }
        }

        {
            let metric = throttling_occurred_family.get_or_create(&ThrottlingOccurredLabels { kind: ThrottlingKind::SoftTemperatureLimit });
            if state.soft_temperature_limit_has_occurred && metric.get() == 0 {
                metric.inc();
            }
        }

        Ok(())
    }
}
