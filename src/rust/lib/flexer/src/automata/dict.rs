use std::ops::Range;
use std::collections::BTreeSet;

#[derive(Clone,Debug)]
pub struct Dict {
    divisions: BTreeSet<i64>
}

impl Dict {
  pub fn insert(&mut self, range: Range<i64>) {
    self.divisions.insert(range.start);
    self.divisions.insert(range.end + 1);
  }

  pub fn len(&self) -> usize {
    self.divisions.len() - 1
  }
}

impl<'a> IntoIterator for &'a Dict {
    type Item = (Range<usize>, usize);
    type IntoIter = Range<Self::Item>; // TODO

    fn into_iter(self) -> Self::IntoIter {
        self.divisions.into_iter().zip(self.divisions.into_iter().skip(1))
            .enumerate()
            .map(|((start, end), ix)| (start..end, ix))
    }
}
