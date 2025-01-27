use std::{cell::RefCell, collections::HashMap, time::{Duration, Instant}};

thread_local! {
    static PROFILER: RefCell<Profiler> = RefCell::new(Profiler::new());
}

const INITIAL_CAPACITY: usize = 2usize.pow(25);

#[derive(Default)]
pub struct Profiler {
    starting_capacity: usize,
    frames: Vec<Frame>,
    current_frame: Vec<Event>
}

impl Profiler {
    fn new() -> Profiler {
        Profiler { starting_capacity: INITIAL_CAPACITY, frames: Vec::new(), current_frame: Vec::with_capacity(INITIAL_CAPACITY) }
    }

    fn call(&mut self, name: &'static str) {
        self.current_frame.push(Event {
            op: Operation::Call(name),
            timestamp: Instant::now()
        });
    }

    fn ret(&mut self) {
        self.current_frame.push(Event {
            op: Operation::Return,
            timestamp: Instant::now()
        });
    }

    fn end_frame(&mut self) {
        if self.current_frame.len() > self.starting_capacity {
            self.starting_capacity = (self.current_frame.len() * 3) / 2;
        }

        let mut frame = Vec::with_capacity(self.starting_capacity);
        std::mem::swap(&mut self.current_frame, &mut frame);

        self.frames.push(Frame { calls: frame.into_boxed_slice() });
    }
}

struct FuncAnalysis {
    name: &'static str,
    calls: u32,
    total_time: Duration,
    average_time: Duration,
    min_time: Duration,
    max_time: Duration
}

pub struct AnalyzedFrames {
    map: HashMap<&'static str, FuncAnalysis>
}

impl AnalyzedFrames {
    fn new(mut profiler: Profiler) -> AnalyzedFrames {
        profiler.end_frame();

        let mut map = HashMap::new();
        for frame in profiler.frames {
            let mut call_stack = Vec::new();
            for Event { op, timestamp } in frame.calls {
                match op {
                    Operation::Call(name) => call_stack.push((name, timestamp)),
                    Operation::Return => {
                        let (name, call_timestamp) = call_stack.pop().unwrap();
                        
                        let v = match map.get_mut(name) {
                            Some(v) => v,
                            None => {
                                map.insert(name, FuncAnalysis {
                                    name,
                                    calls: 0,
                                    total_time: Duration::ZERO,
                                    average_time: Duration::ZERO,
                                    min_time: Duration::MAX,
                                    max_time: Duration::ZERO,
                                });
                                map.get_mut(name).unwrap()
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
            v.average_time = v.total_time / v.calls;
        }

        AnalyzedFrames { map }
    }
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

pub fn profile_current_thread(sort_by: Sort, unit: Unit) {
    let profiler =  PROFILER.take();
    let analysis = AnalyzedFrames::new(profiler);

    let cmp = match sort_by {
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

    let mut fns: Vec<_> = analysis.map.into_iter().collect();
    fns.sort_by(cmp);

    let convert = match unit {
        Unit::Second => |d: &Duration| d.as_secs() as u128,
        Unit::Millisecond => Duration::as_millis,
        Unit::Microsecond => Duration::as_micros,
        Unit::Nanosecond => Duration::as_nanos,
    };

    println!("Function Name        Total Time   Min Time   Max Time   Avg Time                Calls");
    for fn_ in fns {
        println!("{:20} {:10} {:10} {:10} {:10} {:20}",
            fn_.0,
            convert(&fn_.1.total_time),
            convert(&fn_.1.min_time),
            convert(&fn_.1.max_time),
            convert(&fn_.1.average_time),
            fn_.1.calls
        );
    }
}

pub fn init_thread() {
    PROFILER.with_borrow_mut(|profiler| {
        profiler.current_frame.push(Event { op: Operation::Return, timestamp: Instant::now() });
        profiler.current_frame.pop();
    })
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
    calls: Box<[Event]>
}

pub struct FunctionCall {}

impl FunctionCall {
    pub fn new<'a>(name: &'static str) -> FunctionCall {
        PROFILER.with_borrow_mut(|profiler| {
            profiler.call(name);
        });

        FunctionCall {}
    }
}

impl Drop for FunctionCall {
    fn drop(&mut self) {
        PROFILER.with_borrow_mut(|profiler| {
            profiler.ret();
        });
    }
}