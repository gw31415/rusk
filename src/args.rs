use std::env;

/// A custom iterator to parse the arguments.
/// - IntoIterator is implemented as the Iterator of the positional arguments.
pub struct Args {
    iter: PositionalArgsIter,
}

impl Args {
    /// Creates a new Args iterator.
    pub fn new() -> Self {
        let mut inner = env::args();
        inner.next(); // Skip the first argument
        let first = inner.next();
        Self {
            iter: PositionalArgsIter {
                inner,
                first,
                first_read: false,
            },
        }
    }
    /// Whether or not there are no positional arguments.
    pub fn no_pargs(&self) -> bool {
        self.iter.first.is_none()
    }
}

impl IntoIterator for Args {
    type Item = String;
    type IntoIter = PositionalArgsIter;

    fn into_iter(self) -> Self::IntoIter {
        self.iter
    }
}

/// An iterator over the positional arguments.
pub struct PositionalArgsIter {
    inner: env::Args,
    first: Option<String>,
    first_read: bool,
}

impl Iterator for PositionalArgsIter {
    type Item = String;

    fn next(&mut self) -> Option<Self::Item> {
        if !self.first_read {
            self.first_read = true;
            return self.first.take();
        }
        self.inner.next()
    }
}
