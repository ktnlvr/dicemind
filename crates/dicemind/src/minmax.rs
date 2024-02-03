// O(1) max
// O(1) min
// O(logN) insert
// O(logN) prune
#[derive(Debug)]
pub struct MinMax<T: Ord>(Vec<T>);

impl<T: Ord> Default for MinMax<T> {
    fn default() -> Self {
        Self(Default::default())
    }
}

impl<T: Ord> FromIterator<T> for MinMax<T> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        let mut vec: Vec<T> = iter.into_iter().collect();
        vec.sort_unstable();
        Self(vec)
    }
}

impl<T: Ord> MinMax<T> {
    fn vec_mut(&mut self) -> &mut Vec<T> {
        &mut self.0
    }

    fn vec(&self) -> &Vec<T> {
        &self.0
    }

    pub fn into_inner(self) -> Vec<T> {
        assert!(self.0.is_sorted());
        self.0
    }

    pub fn max(&self) -> Option<&T> {
        self.vec().get(self.vec().len() - 1)
    }

    pub fn min(&self) -> Option<&T> {
        self.vec().first()
    }

    pub fn insort(&mut self, value: T) -> usize {
        let (Ok(idx) | Err(idx)) = self.vec().binary_search(&value);
        self.vec_mut().insert(idx, value);
        idx
    }
}

#[cfg(test)]
mod tests {
    use super::MinMax;

    #[test]
    fn random() {
        let random: Vec<i32> = vec![1, 2, 9, 2, -4, 12, 0, -11];
        let minmax = MinMax::from_iter(random.iter().cloned());

        assert!(minmax.vec().is_sorted());
        assert_eq!(minmax.max().cloned(), random.iter().cloned().max());
        assert_eq!(minmax.min().cloned(), random.iter().cloned().min());
    }
}
