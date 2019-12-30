fn test_bounds(buflen: usize, stride: usize, x: usize, y: usize, w: usize, h: usize) {
    // The iterator itself is safe even if these preconditions are not met
    debug_assert!(w > 0);
    debug_assert!(h > 0);
    debug_assert!(x + w <= stride);
    debug_assert!(((y + h - 1) * stride + x + w) <= buflen);
}

/// Iterate through a rectangular area of tiles
///
/// The iterator returns (i, j, &mut T) tuples,
/// where (i, j) is the index of the tile.
pub struct MutableTileIterator<'a, T: 'a> {
    buf: &'a mut [T],
    skip: usize,
    x0: usize,
    x: usize,
    x1: usize,
    row: usize,
    row1: usize,
}

impl<'a, T> MutableTileIterator<'a, T> {
    pub fn new(
        buf: &'a mut [T],
        stride: usize,
        x: usize,
        y: usize,
        w: usize,
        h: usize,
    ) -> MutableTileIterator<'a, T> {
        test_bounds(buf.len(), stride, x, y, w, h);
        MutableTileIterator {
            buf: &mut buf[(y * stride + x)..],
            skip: stride - w + 1,
            x0: x,
            x,
            x1: x + w,
            row: y,
            row1: y + h,
        }
    }
}

impl<'a, T> Iterator for MutableTileIterator<'a, T> {
    type Item = (i32, i32, &'a mut T);
    fn next(&mut self) -> Option<Self::Item> {
        if self.row < self.row1 {
            let (i, j) = (self.x as i32, self.row as i32);

            self.x += 1;
            let skip;
            if self.x >= self.x1 {
                self.x = self.x0;
                self.row += 1;
                if self.row < self.row1 {
                    skip = self.skip;
                } else {
                    skip = 1usize;
                }
            } else {
                skip = 1usize;
            }

            let tmp = std::mem::replace(&mut self.buf, &mut []);
            let (head, tail) = tmp.split_at_mut(skip);
            self.buf = tail;

            Some((i, j, &mut head[0]))
        } else {
            None
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let s = (self.row1 - self.row) * (self.x1 - self.x0);
        (s, Some(s))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mutation_full() {
        #[rustfmt::skip]
        let mut buf = [
            0, 0, 0,
            0, 0, 0,
            0, 0, 0,
        ];
        let iter = MutableTileIterator::new(&mut buf, 3, 0, 0, 3, 3);
        assert_eq!(iter.size_hint(), (9, Some(9)));
        for (i, j, item) in iter {
            *item = j * 10 + i
        }
        #[rustfmt::skip]
        let expected = [
            0,   1,  2,
            10, 11, 12,
            20, 21, 22,
        ];
        assert_eq!(buf, expected);
    }

    #[test]
    fn test_mutation_partial() {
        #[rustfmt::skip]
        let mut buf = [
            0, 0, 0, 0,
            0, 0, 0, 0,
            0, 0, 0, 0,
            0, 0, 0, 0,
        ];
        let iter = MutableTileIterator::new(&mut buf, 4, 1, 1, 2, 2);
        for (i, j, item) in iter {
            *item = j * 10 + i
        }
        #[rustfmt::skip]
        let expected = [
            0,  0,  0, 0,
            0, 11, 12, 0,
            0, 21, 22, 0,
            0,  0, 0,  0,
        ];
        assert_eq!(buf, expected);
    }

    #[test]
    fn test_mutation_partial_end() {
        #[rustfmt::skip]
        let mut buf = [
            0, 0, 0, 0,
            0, 0, 0, 0,
            0, 0, 0, 0,
            0, 0, 0, 0,
        ];
        let iter = MutableTileIterator::new(&mut buf, 4, 2, 2, 2, 2);
        for (i, j, item) in iter {
            *item = j * 10 + i
        }
        #[rustfmt::skip]
        let expected = [
            0,  0,  0,  0,
            0,  0,  0,  0,
            0,  0, 22, 23,
            0,  0, 32, 33,
        ];
        assert_eq!(buf, expected);
    }
}
