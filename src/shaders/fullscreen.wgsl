struct VertexOutput {
  @builtin(position) position : vec4<f32>,
  @location(0) frag_uv : vec2<f32>,
}

@vertex
fn main(
  @location(0) position : vec4<f32>,
  @location(1) uv : vec2<f32>
) -> VertexOutput {
  var output : VertexOutput;
  output.position = position;
  output.frag_uv = uv;
  return output;
}