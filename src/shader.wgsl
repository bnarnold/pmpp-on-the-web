@group(0)
@binding(0)
var<storage, read_write> in_out: array<f32>;

@compute
@workgroup_size(256)
fn main(@builtin(global_invocation_id) gid: vec3<u32>) {
    if gid.x < arrayLength(&in_out) {
        in_out[gid.x] += 10;
    }
}
