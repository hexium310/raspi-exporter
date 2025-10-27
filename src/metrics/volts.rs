use prometheus_client::encoding::{EncodeLabelSet, EncodeLabelValue, LabelValueEncoder};
use strum::Display as StrumDisplay;

#[derive(Clone, Debug, Hash, PartialEq, Eq, EncodeLabelSet)]
pub struct VoltLabels {
    pub kind: VoltKind,
}

#[derive(Clone, PartialEq, Eq, Hash, Debug, StrumDisplay)]
pub enum VoltKind {
    #[strum(to_string = "VC4 core")]
    Core,
    #[strum(to_string = "SDRAM core")]
    SdramC,
    #[strum(to_string = "SDRAM I/O")]
    SdramI,
    #[strum(to_string = "SDRAM PHY")]
    SdramP,
}

impl EncodeLabelValue for VoltKind {
    fn encode(&self, encoder: &mut LabelValueEncoder) -> Result<(), std::fmt::Error> {
        self.to_string().encode(encoder)
    }
}
