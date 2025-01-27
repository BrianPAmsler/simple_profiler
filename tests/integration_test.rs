use simple_profiler::profile_macro::profile;


#[profile]
pub fn idk(a: f32, b: i32) -> usize {
    0
}

#[test]
pub fn test() {
    let v = idk(0.0, 0);

    println!("{}", v);
}