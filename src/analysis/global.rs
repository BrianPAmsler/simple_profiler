use std::collections::HashMap;

use crate::profiler::{Event, Frames, Operation};

use super::{FrameAnalyzer, FuncAnalysis, Sort, Unit};


pub struct GlobalAnalyzer {
    map: HashMap<&'static str, FuncAnalysis>,
    sort_by: Sort,
    unit: Unit
}

impl GlobalAnalyzer {
    pub fn set_sort(&mut self, sort_by: Sort) {
        self.sort_by = sort_by;
    }

    pub fn set_unit(&mut self, unit: Unit) {
        self.unit = unit;
    }
}

impl FrameAnalyzer for GlobalAnalyzer {
    fn new(frames: Frames) -> GlobalAnalyzer {
        let mut map: HashMap<&'static str, FuncAnalysis> = HashMap::new();
        for frame in frames.0 {
            let mut call_stack = Vec::new();
            for Event { op, timestamp } in frame.calls {
                match op {
                    Operation::Call(name) => call_stack.push((name, timestamp)),
                    Operation::Return => {
                        let (name, call_timestamp) = call_stack.pop().unwrap();
                        
                        let v = match map.get_mut(&name.0) {
                            Some(v) => v,
                            None => {
                                map.insert(name.0, FuncAnalysis {
                                    calls: 0,
                                    total_time: 0,
                                    average_time: 0,
                                    min_time: u128::MAX,
                                    max_time: 0,
                                });
                                map.get_mut(&name.0).unwrap()
                            },
                        };

                        let duration = timestamp - call_timestamp;
                        v.calls += 1;
                        v.total_time += duration;

                        if duration > v.max_time {
                            v.max_time = duration;
                        }

                        if duration < v.min_time {
                            v.min_time = duration;
                        }
                    },
                }
            }
        }

        for (_, v) in &mut map {
            v.average_time = v.total_time / v.calls as u128;
        }

        GlobalAnalyzer { map, sort_by: Sort::TotalTime, unit: Unit::Millisecond  }
    }

    fn analyze(self, fmt: &mut dyn std::io::Write) -> std::io::Result<()> {
        let cmp = match self.sort_by {
            Sort::NameAscending => |a: &(&str, FuncAnalysis) , b: &(&str, FuncAnalysis) | {
                a.0.cmp(b.0)
            },
            Sort::NameDescending => |a: &(&str, FuncAnalysis) , b: &(&str, FuncAnalysis) | {
                a.0.cmp(b.0).reverse()
            },
            Sort::TotalTime => |a: &(&str, FuncAnalysis) , b: &(&str, FuncAnalysis) | {
                a.1.total_time.cmp(&b.1.total_time).reverse()
            },
            Sort::MinTime => |a: &(&str, FuncAnalysis) , b: &(&str, FuncAnalysis) | {
                a.1.min_time.cmp(&b.1.min_time).reverse()
            },
            Sort::MaxTime => |a: &(&str, FuncAnalysis) , b: &(&str, FuncAnalysis) | {
                a.1.max_time.cmp(&b.1.max_time).reverse()
            },
            Sort::AverageTime => |a: &(&str, FuncAnalysis) , b: &(&str, FuncAnalysis) | {
                a.1.average_time.cmp(&b.1.average_time).reverse()
            },
            Sort::Calls => |a: &(&str, FuncAnalysis) , b: &(&str, FuncAnalysis) | {
                a.1.calls.cmp(&b.1.calls).reverse()
            },
        };
    
        let mut fns: Vec<_> = self.map.into_iter().collect();
        fns.sort_by(cmp);
    
        let convert = |time: u128| -> u128 {
            match self.unit {
                Unit::Second => time / 1_000_000_000,
                Unit::Millisecond => time / 1_000_000,
                Unit::Microsecond => time / 1_000,
                Unit::Nanosecond => time,
            }
        };
    
        writeln!(fmt, "Function Name        Total Time   Min Time   Max Time   Avg Time                Calls")?;
        for fn_ in fns {
            writeln!(fmt, "{:20} {:10} {:10} {:10} {:10} {:20}",
                fn_.0,
                convert(fn_.1.total_time),
                convert(fn_.1.min_time),
                convert(fn_.1.max_time),
                convert(fn_.1.average_time),
                fn_.1.calls
            )?;
        }

        Ok(())
    }
}