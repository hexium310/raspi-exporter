pub mod throttled;
pub mod volts;

pub trait Parser {
    type Item<'a> where Self: 'a;

    fn parse<'a>(&'a self, input: &'a str) -> anyhow::Result<Self::Item<'a>>;
}
