use std::collections::VecDeque;

pub trait VecExt {
    /// Removes a list of indices from a Vector.
    /// Assumes that the provided indices are already in sorted order.
    fn remove_indices(&mut self, to_remove: VecDeque<usize>);
}

impl<T> VecExt for Vec<T> {
    fn remove_indices(&mut self, mut to_remove: VecDeque<usize>) {
        let mut i = 0;

        let mut next_remove = to_remove.pop_front();

        self.retain(|_| {
            let keep = if let Some(next_remove) = next_remove {
                i != next_remove
            } else {
                true
            };

            if !keep {
                next_remove = to_remove.pop_front();
            }

            i += 1;
            keep
        });
    }
}

#[cfg(test)]
mod tests {
    use std::collections::VecDeque;

    use crate::vec_ext::VecExt;

    #[test]
    fn removes_requested_indices() {
        let mut data: Vec<i32> = (0..10).collect();
        let remove: VecDeque<usize> = vec![1, 4, 6].into_iter().collect();

        data.remove_indices(remove);

        assert_eq!(data, vec![0, 2, 3, 5, 7, 8, 9])
    }
}
