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

#[test]
pub fn test() {
    for _ in 0..1000000 {
        let a = 5.0;
        let b = 12.6;

        mul(a, b);
        add(a, b);
        sub(a, b);
        div(a, b);
    }

    profiler::profile_current_thread(simple_profiler::analysis::Sort::TotalTime, simple_profiler::analysis::Unit::Nanosecond);
    panic!()
}