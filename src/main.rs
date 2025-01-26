use pmpp_on_the_web::{init_logging, run_shader};

fn main() {
    init_logging();
    let shader = include_str!("shader.wgsl");
    const N: usize = 10_000;
    let input: Vec<_> = (0..N).map(|i| i as f32).collect();
    let result: Vec<f32> = pollster::block_on(run_shader(
        shader,
        (input.as_slice(),),
        (N.next_multiple_of(256) as u32, 1, 1),
        N,
    ))
    .unwrap();
    println!("Got {} elements", result.len());
    println!("Here are the first 20: {:?}", &result[..20]);
}
