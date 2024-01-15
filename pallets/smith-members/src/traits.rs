use crate::SmithRemovalReason;

pub trait OnSmithDelete<IdtyIndex> {
    fn on_smith_delete(idty_index: IdtyIndex, reason: SmithRemovalReason);
}
impl<IdtyIndex> OnSmithDelete<IdtyIndex> for () {
    fn on_smith_delete(_: IdtyIndex, _: SmithRemovalReason) {}
}
