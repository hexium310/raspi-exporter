pub mod throttled;

pub trait Parser {
    type Item;

    fn parse(&self, input: &str) -> anyhow::Result<Self::Item>;
}
