/// Merges two decisions into a single decision.
use crate::system::schedule::claim::decision::{
    Append,
    Cut,
};

pub trait Merger {
    type Decision;
}

impl Merger for (Cut, Cut) {
    type Decision = Cut;
}

impl Merger for (Cut, Append) {
    type Decision = Cut;
}

impl Merger for (Append, Cut) {
    type Decision = Cut;
}

impl Merger for (Append, Append) {
    type Decision = Append;
}
