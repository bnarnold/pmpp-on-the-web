@group(0) @binding(0) var<storage, read_write> lhs: array<f32>;
@group(0) @binding(1) var<storage, read_write> lhs_width: u32;
@group(0) @binding(2) var<storage, read_write> rhs: array<f32>;
@group(0) @binding(3) var<storage, read_write> rhs_width: u32;
@group(1) @binding(0) var<storage, read_write> out: array<f32>;

@compute
@workgroup_size(32,1,1)
fn mmul(@builtin(global_invocation_id) gid: vec3<u32>) {
    for (var j = 0u; j < rhs_width; j += 1) {
        for (var k = 0u; k < lhs_width; k += 1) {
            out[gid.x * rhs_width + j] += lhs[gid.x * lhs_width + k] * rhs[k * rhs_width + j];
        }
    }
}


