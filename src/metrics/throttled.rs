use prometheus_client::encoding::{EncodeLabelSet, EncodeLabelValue, LabelValueEncoder};
use strum::Display as StrumDisplay;

#[derive(Clone, Debug, Hash, PartialEq, Eq, EncodeLabelSet)]
pub struct ThrottlingActiveLabels {
    pub kind: ThrottlingKind,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq, EncodeLabelSet)]
pub struct ThrottlingOccurredLabels {
    pub kind: ThrottlingKind,
}

#[derive(Clone, PartialEq, Eq, Hash, Debug, StrumDisplay)]
pub enum ThrottlingKind {
    #[strum(to_string = "undervoltage")]
    Undervoltage,
    #[strum(to_string = "arm frequency")]
    ArmFrequency,
    #[strum(to_string = "throttled")]
    Throttled,
    #[strum(to_string = "soft temperature limit")]
    SoftTemperatureLimit,
}

impl EncodeLabelValue for ThrottlingKind {
    fn encode(&self, encoder: &mut LabelValueEncoder) -> Result<(), std::fmt::Error> {
        self.to_string().encode(encoder)
    }
}
