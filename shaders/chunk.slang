import include.camera;

[vk::binding(0, 0)]
ConstantBuffer<Camera> camera;

struct Chunk
{
    float x;
    float y;
    float z;
}

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
ConstantBuffer<Chunk> chunk;

[vk::binding(1, 1)]
StructuredBuffer<Face> faces;

struct VertexOutput
{
    float4 clip_position : SV_Position;
    uint instance_id;
    float2 uv;
    float3 normal;
}

[shader("vertex")]
VertexOutput vertex(uint vertex_index: SV_VertexID, uint instance_id: SV_InstanceID)
{
    var out : VertexOutput;
    out.instance_id = instance_id;

    let chunk_position = float3(chunk.x, chunk.y, chunk.z);

    let face = faces[instance_id];
    let position = float3(face.x, face.y, face.z);
    let size = float2(face.width, face.height);

    let x = float((vertex_index >> 0) & 1);
    let y = float((vertex_index >> 1) & 1);
    out.uv = float2(x, y);

    let offset_2d = (out.uv - 0.5) * size;
    var offset : float3;
    switch (vertex_index >> 3)
    {
    case 0:
        offset = float3(0.0, offset_2d.x, offset_2d.y);
        out.normal = float3(1.0, 0.0, 0.0);
        break;
    case 1:
        offset = float3(0.0, offset_2d.x, offset_2d.y);
        out.normal = float3(-1.0, 0.0, 0.0);
        break;
    case 2:
        offset = float3(offset_2d.x, 0.0, offset_2d.y);
        out.normal = float3(0.0, 1.0, 0.0);
        break;
    case 3:
        offset = float3(offset_2d.x, 0.0, offset_2d.y);
        out.normal = float3(0.0, -1.0, 0.0);
        break;
    case 4:
        offset = float3(offset_2d.x, offset_2d.y, 0.0);
        out.normal = float3(0.0, 0.0, 1.0);
        break;
    case 5:
        offset = float3(offset_2d.x, offset_2d.y, 0.0);
        out.normal = float3(0.0, 0.0, -1.0);
        break;
    }

    var point = chunk_position + position + offset;
    point = camera.transform.reverse().transform_point(point);
    out.clip_position = float4(point.z / camera.aspect, point.y, camera.near_plane, point.x);

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

    let light_direction = normalize(float3(0.2, 0.8, 0.1));
    let light = max(0.1, dot(in.normal, light_direction) * 0.5 + 0.5);

    out.color = float4(color * light, 1.0);

    return out;
}
