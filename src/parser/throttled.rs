use anyhow::Context as _;

use crate::parser::Parser;

#[derive(Debug)]
pub struct ThrottledParser;

// https://www.raspberrypi.com/documentation/computers/os.html#get_throttled
#[derive(Debug, Default, PartialEq, Eq)]
pub struct ThrottledState {
    pub undervoltage_detected: bool,
    pub arm_frequency_capped: bool,
    pub currently_throttled: bool,
    pub soft_temperature_limit_active: bool,
    pub undervoltage_has_occurred: bool,
    pub arm_frequency_capping_has_occurred: bool,
    pub throttling_has_occurred: bool,
    pub soft_temperature_limit_has_occurred: bool,
}

impl Parser for ThrottledParser {
    type Item = ThrottledState;

    fn parse(&self, input: &str) -> anyhow::Result<Self::Item> {
        let invalid_input_error = || format!("invalid input: {input}");

        let decimal = input
            .trim()
            .split_once('=')
            .with_context(invalid_input_error)
            .and_then(|(_, v)| u32::from_str_radix(&v[2..], 16).map_err(|_| anyhow::anyhow!(invalid_input_error())))?;

        let state = Self::Item {
            undervoltage_detected: decimal & 0b1 << 0 != 0,
            arm_frequency_capped: decimal & 0b1 << 1 != 0,
            currently_throttled: decimal & 0b1 << 2 != 0,
            soft_temperature_limit_active: decimal & 0b1 << 3 != 0,
            undervoltage_has_occurred: decimal & 0b1 << 16 != 0,
            arm_frequency_capping_has_occurred: decimal & 0b1 << 17 != 0,
            throttling_has_occurred: decimal & 0b1 << 18 != 0,
            soft_temperature_limit_has_occurred: decimal & 0b1 << 19 != 0,
        };

        Ok(state)
    }
}

#[cfg(test)]
mod tests {
    use crate::parser::{throttled::{ThrottledParser, ThrottledState}, Parser};

    #[test]
    fn parse() {
        let throttled_parser = ThrottledParser;
        let result = throttled_parser.parse("throttled=0xd0005").unwrap();

        assert_eq!(
            result,
            ThrottledState {
                undervoltage_detected: true,
                arm_frequency_capped: false,
                currently_throttled: true,
                soft_temperature_limit_active: false,
                undervoltage_has_occurred: true,
                arm_frequency_capping_has_occurred: false,
                throttling_has_occurred: true,
                soft_temperature_limit_has_occurred: true,
            }
        )
    }
}
