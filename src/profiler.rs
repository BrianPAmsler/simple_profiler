use std::{cell::RefCell, fs::OpenOptions, io::{stdout, Read, Write}, time::Instant};

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::{analysis::{global::GlobalAnalyzer, FrameAnalyzer, Sort, Unit}, serialization::StaticStr};

thread_local! {
    static PROFILER: RefCell<Profiler> = RefCell::new(Profiler::new());
}

const INITIAL_CAPACITY: usize = 2usize.pow(25);

pub struct Profiler {
    initial_time: Instant,
    starting_capacity: usize,
    frames: Vec<Frame>,
    current_frame: Vec<Event>
}

impl Default for Profiler {
    fn default() -> Self {
        Self { initial_time: Instant::now(), starting_capacity: Default::default(), frames: Default::default(), current_frame: Default::default() }
    }
}

impl Profiler {
    fn new() -> Profiler {
        Profiler { initial_time: Instant::now(), starting_capacity: INITIAL_CAPACITY, frames: Vec::new(), current_frame: Vec::with_capacity(INITIAL_CAPACITY) }
    }

    fn call(&mut self, name: &'static str) {
        self.current_frame.push(Event {
            op: Operation::Call(StaticStr(name)),
            timestamp: (Instant::now() - self.initial_time).as_nanos()
        });
    }

    fn ret(&mut self) {
        self.current_frame.push(Event {
            op: Operation::Return,
            timestamp: (Instant::now() - self.initial_time).as_nanos()
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

    pub fn get_frames(mut self) -> Frames {
        self.end_frame();
        Frames(self.frames.into_boxed_slice())
    }

    pub fn dump_frames<P: AsRef<std::path::Path>>(mut self, path: P) -> Result<(), Error> {
        let mut file = OpenOptions::new()
            .write(true)
            .create(true)
            .open(path)?;
        
        self.end_frame();

        let bytes = postcard::to_extend(&self.frames, Vec::new()).unwrap();

        file.write(&bytes)?;

        Ok(())
    }

    pub fn load_frames<P: AsRef<std::path::Path>>(path: P) -> Result<Frames, Error> {
        let mut file = OpenOptions::new()
            .write(true)
            .create(true)
            .open(path)?;

        let mut bytes = Vec::new();
        file.read_to_end(&mut bytes)?;

        Ok(Frames(postcard::from_bytes(&bytes)?))
    }
}

pub struct Frames(pub(in crate) Box<[Frame]>);

#[derive(Error, Debug)]
pub enum Error {
    #[error("{0}")]
    SerializationError(#[from] postcard::Error),
    #[error("{0}")]
    IoError(#[from] std::io::Error)
}

pub fn profile_current_thread(sort_by: Sort, unit: Unit) {
    let profiler =  PROFILER.take();
    let mut analysis = GlobalAnalyzer::new(profiler.get_frames());
    analysis.set_sort(sort_by);
    analysis.set_unit(unit);

    analysis.analyze(&mut stdout()).unwrap();
}

#[derive(Serialize, Deserialize, Clone, Copy)]
pub(in crate) enum Operation {
    Call(StaticStr),
    Return
}

#[derive(Serialize, Deserialize)]
pub(in crate) struct Event {
    pub op: Operation,
    pub timestamp: u128
}

#[derive(Serialize, Deserialize)]
pub(in crate) struct Frame {
    pub calls: Box<[Event]>
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