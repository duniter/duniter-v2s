// tmp

use frame_support::weights::Weight;

pub trait WeightInfo {
    fn queue_refund() -> Weight;
    fn spend_quota() -> Weight;
    fn try_refund() -> Weight;
    fn do_refund() -> Weight;
    fn on_process_refund_queue() -> Weight;
    fn on_process_refund_queue_elements(_i: u32) -> Weight;
}

impl WeightInfo for () {
    fn queue_refund() -> Weight {
        Weight::from_parts(100u64, 0)
    }

    fn spend_quota() -> Weight {
        Weight::from_parts(25u64, 0)
    }

    fn try_refund() -> Weight {
        Weight::from_parts(100u64, 0)
    }

    fn do_refund() -> Weight {
        Weight::from_parts(25u64, 0)
    }

    fn on_process_refund_queue() -> Weight {
        Weight::from_parts(1u64, 0)
    }

    fn on_process_refund_queue_elements(_i: u32) -> Weight {
        Weight::from_parts(1u64, 0)
    }
}
