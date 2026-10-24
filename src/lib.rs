// Re-export for public use under the crate's root.
pub use crate::_cogs::resources::Resource;
pub use crate::_cogs::selectors::Selector;

mod _cogs;

/// Optionally pre-heat certain parts of the library,
/// so that pre-heating does not happen lazily in some of the consuming threads.
///
/// # Examples
/// ```
/// use example1::init as example1_init;
///
/// fn main() {
///     example1_init();
///     // do the multi-threaded stuff…
/// }
/// ```
pub fn init() {
    {
        use crate::_cogs::selectors::K8S_VERSION_PATTERN;
        let _ = &*K8S_VERSION_PATTERN;
    }
}
