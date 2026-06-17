#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PageRequest {
    pub limit: u32,
    pub offset: u64,
}

impl PageRequest {
    pub const DEFAULT_LIMIT: u32 = 50;
    pub const MAX_LIMIT: u32 = 500;

    #[must_use]
    pub const fn new(limit: u32, offset: u64) -> Self {
        Self { limit, offset }
    }

    #[must_use]
    pub const fn first_page() -> Self {
        Self {
            limit: Self::DEFAULT_LIMIT,
            offset: 0,
        }
    }

    #[must_use]
    pub fn clamped(self) -> Self {
        let limit = if self.limit == 0 {
            Self::DEFAULT_LIMIT
        } else {
            self.limit.min(Self::MAX_LIMIT)
        };

        Self {
            limit,
            offset: self.offset,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::PageRequest;

    #[test]
    fn clamped_uses_default_limit_for_zero_and_caps_large_limits() {
        assert_eq!(
            PageRequest::new(0, 7).clamped(),
            PageRequest::new(PageRequest::DEFAULT_LIMIT, 7)
        );
        assert_eq!(
            PageRequest::new(PageRequest::MAX_LIMIT + 1, 7).clamped(),
            PageRequest::new(PageRequest::MAX_LIMIT, 7)
        );
    }
}
