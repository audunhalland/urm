pub trait Constrain {}

pub struct Filter {
    pub range: std::ops::Range<Option<usize>>,
}

pub trait Range: Send + Sync + 'static {
    fn offset(&self) -> Option<usize> {
        None
    }

    fn limit(&self) -> Option<usize> {
        None
    }
}

impl Range for std::ops::Range<usize> {
    fn offset(&self) -> Option<usize> {
        Some(self.start)
    }

    fn limit(&self) -> Option<usize> {
        Some(self.end)
    }
}

impl Range for std::ops::Range<Option<usize>> {
    fn offset(&self) -> Option<usize> {
        self.start
    }

    fn limit(&self) -> Option<usize> {
        self.end
    }
}
