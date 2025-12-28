#![feature(stmt_expr_attributes, proc_macro_hygiene)]

use profile_macro::profile;
use simple_profiler::profiler;

#[profile]
pub fn mul(a: f32, b: f32) -> f32 {
    a * b
}

#[profile]
pub fn add(a: f32, b: f32) -> f32 {
    a + b
}

#[profile]
pub fn sub(a: f32, b: f32) -> f32 {
    a - b
}

#[profile]
pub fn div(a: f32, b: f32) -> f32 {
    a / b
}

#[profile]
pub fn vec<F: Fn(f32, f32) -> f32>(a: &[f32], b: &[f32], op: F) -> Vec<f32> {
    let mut out = Vec::new();
    out.reserve(a.len());

    for (a, b) in a.iter().zip(b.iter()) {
        out.push(op(*a, *b));
    }

    out
}

#[test]
pub fn test() {
    const COUNT: usize = 1000000;

    let mut vec_a = Vec::new();
    let mut vec_b = Vec::new();

    vec_a.reserve(COUNT);
    vec_b.reserve(COUNT);
    for _ in 0..COUNT {
        let a = 5.0;
        let b = 12.6;

        vec_a.push(a);
        vec_b.push(b);

        #[profile(name = "all")]
        {
            mul(a, b);
            add(a, b);
            sub(a, b);
            div(a, b);
        }
    }

    vec(&vec_a, &vec_b, |a, b| a + b);

    profiler::profile_current_thread(simple_profiler::analysis::Sort::TotalTime, simple_profiler::analysis::Unit::Nanosecond);
    panic!()
}