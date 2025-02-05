pub mod global;

use std::io::Write;

use crate::profiler::Frames;

pub trait FrameAnalyzer {
    fn analyze(self, writer: &mut dyn Write) -> std::io::Result<()>;
    fn new(frames: Frames) -> Self;
}

pub(in crate) struct FuncAnalysis {
    pub calls: u32,
    pub total_time: u128,
    pub average_time: u128,
    pub min_time: u128,
    pub max_time: u128
}

pub enum Sort {
    NameAscending,
    NameDescending,
    TotalTime,
    MinTime,
    MaxTime,
    AverageTime,
    Calls
}

pub enum Unit {
    Second,
    Millisecond,
    Microsecond,
    Nanosecond
}