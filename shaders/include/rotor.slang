struct Rotor
{
    float s;
    float e1e2;
    float e1e3;
    float e2e3;

    Rotor reverse()
    {
        Rotor result;
        result.s = this.s;
        result.e1e2 = -this.e1e2;
        result.e1e3 = -this.e1e3;
        result.e2e3 = -this.e2e3;
        return result;
    }

    float3 rotate_vector(float3 point)
    {
        let _0 = this.s;
        let _1 = this.e1e2;
        let _2 = this.e1e3;
        let _3 = this.e2e3;
        let _4 = point.x;
        let _5 = point.y;
        let _6 = point.z;

        float3 result;
        result.x = (_4) + (-2.0 * _0 * _1 * _5) + (-2.0 * _0 * _2 * _6) + (-2.0 * _1 * _1 * _4) + (-2.0 * _2 * _2 * _4) + (-2.0 * _2 * _3 * _5) + (2.0 * _1 * _3 * _6);
        result.y = (_5) + (-2.0 * _0 * _3 * _6) + (-2.0 * _1 * _1 * _5) + (-2.0 * _1 * _2 * _6) + (-2.0 * _2 * _3 * _4) + (-2.0 * _3 * _3 * _5) + (2.0 * _0 * _1 * _4);
        result.z = (_6) + (-2.0 * _1 * _2 * _5) + (-2.0 * _2 * _2 * _6) + (-2.0 * _3 * _3 * _6) + (2.0 * _0 * _2 * _4) + (2.0 * _0 * _3 * _5) + (2.0 * _1 * _3 * _4);
        return result;
    }
}
