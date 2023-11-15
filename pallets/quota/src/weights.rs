// tmp

use frame_support::weights::Weight;

pub trait WeightInfo {
    fn queue_refund() -> Weight;
    fn spend_quota() -> Weight;
    fn try_refund() -> Weight;
    fn do_refund() -> Weight;
}

impl WeightInfo for () {
    fn queue_refund() -> Weight {
        Weight::from_parts(999u64, 0)
    }
    fn spend_quota() -> Weight {
        Weight::from_parts(999u64, 0)
    }
    fn try_refund() -> Weight {
        Weight::from_parts(999u64, 0)
    }
    fn do_refund() -> Weight {
        Weight::from_parts(999u64, 0)
    }
}
