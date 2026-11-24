//! How often something is so that does not have to be.

/// How often something is so that does not have to be.
#[derive(Default)]
pub(crate) struct Share {
    looked_at: usize,
    agreed: usize,
}

impl Share {
    pub(crate) fn add(&mut self, agrees: bool) {
        self.looked_at += 1;
        self.agreed += usize::from(agrees);
    }

    /// Prints the share, and checks that it is no lower than `percent`.
    pub(crate) fn check(&self, what: &str, percent: usize) {
        let Self { looked_at, agreed } = self;
        println!("{what}: {agreed} of {looked_at}");
        assert!(
            agreed * 100 >= looked_at * percent,
            "{what} are fewer than they were"
        );
    }
}
