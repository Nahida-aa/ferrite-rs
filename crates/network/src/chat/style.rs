/// Placeholder for `Style.java` (full port pending).  
/// Only supports the minimal interface required by `FormattedText`.
pub struct Style;

impl Style {
    pub const EMPTY: Self = Style;

    /// Merge `self` onto `other`: non-null fields in `self` override `other`.
    /// Currently a no-op placeholder.
    pub fn apply_to(&self, _other: &Self) -> Self {
        Style
    }
}
