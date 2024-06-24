use crate::SmithRemovalReason;

/// Trait for handling actions when a Smith is deleted.
pub trait OnSmithDelete<IdtyIndex> {
    /// Handle the deletion of a smith.
    fn on_smith_delete(idty_index: IdtyIndex, reason: SmithRemovalReason);
}

impl<IdtyIndex> OnSmithDelete<IdtyIndex> for () {
    fn on_smith_delete(_: IdtyIndex, _: SmithRemovalReason) {}
}
