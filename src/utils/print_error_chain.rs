use std::fmt::{self, Display};

pub struct PrintErrorChain<'a>(pub &'a dyn std::error::Error);

impl<'a> Display for PrintErrorChain<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        std::fmt::Display::fmt(self.0, f)?;
        let mut source = self.0.source();
        while let Some(error) = source {
            f.write_str(": ")?;
            std::fmt::Display::fmt(error, f)?;
            source = error.source();
        }
        Ok(())
    }
}
