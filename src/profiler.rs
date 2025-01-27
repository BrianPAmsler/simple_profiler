use std::time::Instant;

pub struct Profiler {
    frames: Vec<Frame>
}

enum Operation {
    Call(&'static str),
    Return
}

struct Event {
    op: Operation,
    timestamp: Instant
}

struct Frame {
    stack: Vec<Event>
}