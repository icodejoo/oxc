//! Stand-in for the napi `ExternalServices` in the pure Rust build.
//!
//! There is no JS side here, so nothing needs setting up and nothing is delegated;
//! it only lets the LSP and Stdin entry points share one code path with the napi build.

/// No-op external services for the pure Rust build.
#[derive(Clone, Default)]
pub struct ExternalServices;

impl ExternalServices {
    /// Nothing to set up without a JS side.
    ///
    /// # Errors
    /// Never fails; the signature matches the napi build.
    #[expect(clippy::unused_self, clippy::unnecessary_wraps)]
    pub fn init(&self, _num_of_threads: usize) -> Result<(), String> {
        Ok(())
    }

    /// Same constructor name as the napi build's test double.
    #[cfg(test)]
    pub fn dummy() -> Self {
        Self
    }
}
