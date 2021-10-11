pub struct FlatIntersperse<Iter, Sep>
where
    Iter: Iterator,
    Iter::Item: IntoIterator,
{
    inner: Option<<Iter::Item as IntoIterator>::IntoIter>,
    iter: Iter,
    sep: Sep,
    need_sep: bool,
}

impl<Iter, Sep> FlatIntersperse<Iter, Sep>
where
    Iter: Iterator,
    Iter::Item: IntoIterator,
{
    pub fn new(iter: Iter, sep: Sep) -> Self {
        Self {
            inner: None,
            iter,
            sep,
            need_sep: false,
        }
    }
}

pub fn flat_intersperse<Iter, Sep>(iter: Iter, sep: Sep) -> FlatIntersperse<Iter::IntoIter, Sep>
where
    Iter: IntoIterator,
    <Iter::IntoIter as Iterator>::Item: IntoIterator,
{
    FlatIntersperse::new(iter.into_iter(), sep)
}

impl<Iter, Sep> Iterator for FlatIntersperse<Iter, Sep>
where
    Iter: Iterator,
    Iter::Item: IntoIterator<Item = Sep>,
    <Iter::Item as IntoIterator>::IntoIter: Iterator<Item = Sep>,
    Sep: Clone,
{
    type Item = Sep;

    fn next(&mut self) -> Option<Sep> {
        loop {
            if let Some(ref mut inner) = self.inner {
                if let Some(item) = inner.next() {
                    return Some(item);
                } else {
                    self.inner = None;
                }
            } else {
                if let Some(inner) = self.iter.next() {
                    self.inner = Some(inner.into_iter());
                    if !self.need_sep {
                        self.need_sep = true;
                    } else {
                        return Some(self.sep.clone());
                    }
                } else {
                    return None;
                }
            }
        }
    }
}

#[cfg(test)]
#[test]
fn flat_maps() {
    let iter = FlatIntersperse::new(vec![vec![1, 2], vec![3], vec![], vec![4]].into_iter(), 42);
    assert_eq!(iter.collect::<Vec<_>>(), vec![1, 2, 42, 3, 42, 42, 4]);
}
