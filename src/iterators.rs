use std::iter::Peekable;

pub trait IdentifyLast: Iterator + Sized {
    fn identify_last(self) -> IdentifyLastIter<Self>;
}

impl<It> IdentifyLast for It
where
    It: Iterator,
{
    fn identify_last(self) -> IdentifyLastIter<Self> {
        IdentifyLastIter(self.peekable())
    }
}

pub struct IdentifyLastIter<It>(Peekable<It>)
where
    It: Iterator;

impl<It> Iterator for IdentifyLastIter<It>
where
    It: Iterator,
{
    type Item = (bool, It::Item);

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().map(|value| (self.0.peek().is_none(), value))
    }
}
