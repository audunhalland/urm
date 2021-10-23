use crate::Table;

pub trait Filter<T: Table>: Send + Sync + 'static {
    fn offset(&self) -> Option<usize> {
        None
    }

    fn limit(&self) -> Option<usize> {
        None
    }
}

impl<T: Table> Filter<T> for std::ops::Range<usize> {
    fn offset(&self) -> Option<usize> {
        Some(self.start)
    }

    fn limit(&self) -> Option<usize> {
        Some(self.end)
    }
}

impl<T: Table> Filter<T> for std::ops::Range<Option<usize>> {
    fn offset(&self) -> Option<usize> {
        self.start
    }

    fn limit(&self) -> Option<usize> {
        self.end
    }
}

impl<T: Table, F0, F1> Filter<T> for (F0, F1)
where
    F0: Filter<T>,
    F1: Filter<T>,
{
    fn offset(&self) -> Option<usize> {
        self.0.offset().or(self.1.offset())
    }

    fn limit(&self) -> Option<usize> {
        self.0.limit().or(self.1.limit())
    }
}
