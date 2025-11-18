import include.camera;

[vk::binding(0, 0)]
ConstantBuffer<Camera> camera;

struct Face
{
    float x;
    float y;
    float z;
    float red;
    float green;
    float blue;
    float width;
    float height;
}

[vk::binding(0, 1)]
StructuredBuffer<Face> faces;

struct VertexOutput
{
    float4 clip_position : SV_Position;
    uint instance_id;
    float2 uv;
}

[shader("vertex")]
VertexOutput vertex(uint vertex_index: SV_VertexID, uint instance_id: SV_InstanceID)
{
    var out : VertexOutput;
    out.instance_id = instance_id;

    let camera_position = float3(camera.x, camera.y, camera.z);

    let face = faces[instance_id];
    let position = float3(face.x, face.y, face.z);
    let size = float2(face.width, face.height);

    let x = float((vertex_index >> 0) & 1);
    let y = float((vertex_index >> 1) & 1);
    out.uv = float2(x, y);

    var point = (position - camera_position) + float3(0.0, (out.uv - 0.5) * size);
    point.z /= camera.aspect;
    out.clip_position = float4(point.zy, camera.near_plane, point.x);

    return out;
}

struct FragmentOutput
{
    float4 color : SV_Target;
}

[shader("fragment")]
FragmentOutput fragment(VertexOutput in)
{
    var out : FragmentOutput;

    let face = faces[in.instance_id];
    let color = float3(face.red, face.green, face.blue);
    out.color = float4(color, 1.0);

    return out;
}
