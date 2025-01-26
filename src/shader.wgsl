@group(0)
@binding(0)
var<storage, read_write> in: array<f32>;

@group(1)
@binding(0)
var<storage, read_write> out: array<f32>;

@compute
@workgroup_size(256)
fn main(@builtin(global_invocation_id) gid: vec3<u32>) {
    if gid.x < arrayLength(&in) {
        out[gid.x] = in[gid.x] + 10;
    }
}
