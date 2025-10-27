use anyhow::Context as _;
use lazy_regex::{lazy_regex, Lazy};
use regex_lite::Regex;

use crate::parser::Parser;

static VOLT_REGEX: Lazy<Regex> = lazy_regex!(r"volt=(?<volt>\d+\.\d+)V");

#[derive(Debug)]
pub struct VoltsParser;

// https://www.raspberrypi.com/documentation/computers/os.html#get_volts
#[derive(Debug, Default, PartialEq)]
pub struct VoltsState {
    pub core: f64,
    pub sdram_c: f64,
    pub sdram_i: f64,
    pub sdram_p: f64,
}

impl Parser for VoltsParser {
    type Item<'a> = &'a str;

    fn parse<'a>(&'a self, input: &'a str) -> anyhow::Result<Self::Item<'a>> {
        let invalid_input_error = || format!("invalid input: {input}");

        let volts = VOLT_REGEX
            .captures(input)
            .and_then(|captures| captures.name("volt"))
            .with_context(invalid_input_error)?
            .as_str();

        Ok(volts)
    }
}

impl TryFrom<Vec<&str>> for VoltsState {
    type Error = anyhow::Error;

    fn try_from(value: Vec<&str>) -> Result<Self, Self::Error> {
        let state = VoltsState {
            core: value[0].parse::<f64>()?,
            sdram_c: value[1].parse::<f64>()?,
            sdram_i: value[2].parse::<f64>()?,
            sdram_p: value[3].parse::<f64>()?,
        };

        Ok(state)
    }
}

#[cfg(test)]
mod tests {
    use crate::parser::{volts::VoltsParser, Parser};

    #[test]
    fn parse() {
        let throttled_parser = VoltsParser;
        let result = throttled_parser.parse("volt=1.3563V").unwrap();

        assert_eq!(result, "1.3563");
    }
}
