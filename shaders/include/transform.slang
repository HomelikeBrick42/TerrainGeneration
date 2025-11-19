import rotor;

struct Transform
{
    float s;
    float e0e1;
    float e0e2;
    float e0e3;
    float e1e2;
    float e1e3;
    float e2e3;
    float e0e1e2e3;

    Transform reverse()
    {
        Transform result;
        result.s = this.s;
        result.e0e1 = -this.e0e1;
        result.e0e2 = -this.e0e2;
        result.e0e3 = -this.e0e3;
        result.e1e2 = -this.e1e2;
        result.e1e3 = -this.e1e3;
        result.e2e3 = -this.e2e3;
        result.e0e1e2e3 = this.e0e1e2e3;
        return result;
    }

    float3 transform_normal(float3 point)
    {
        let _0 = this.s;
        let _1 = this.e0e1;
        let _2 = this.e0e2;
        let _3 = this.e0e3;
        let _4 = this.e1e2;
        let _5 = this.e1e3;
        let _6 = this.e2e3;
        let _7 = this.e0e1e2e3;
        let _8 = point.x;
        let _9 = point.y;
        let _10 = point.z;

        float3 result;
        result.x = (_8) + (-2.0 * _0 * _10 * _5) + (-2.0 * _0 * _4 * _9) + (-2.0 * _4 * _4 * _8) + (-2.0 * _5 * _5 * _8) + (-2.0 * _5 * _6 * _9) + (2.0 * _10 * _4 * _6);
        result.y = (_9) + (-2.0 * _0 * _10 * _6) + (-2.0 * _10 * _4 * _5) + (-2.0 * _4 * _4 * _9) + (-2.0 * _5 * _6 * _8) + (-2.0 * _6 * _6 * _9) + (2.0 * _0 * _4 * _8);
        result.z = (_10) + (-2.0 * _10 * _5 * _5) + (-2.0 * _10 * _6 * _6) + (-2.0 * _4 * _5 * _9) + (2.0 * _0 * _5 * _8) + (2.0 * _0 * _6 * _9) + (2.0 * _4 * _6 * _8);
        return result;
    }

    float3 transform_point(float3 point)
    {
        let _0 = this.s;
        let _1 = this.e0e1;
        let _2 = this.e0e2;
        let _3 = this.e0e3;
        let _4 = this.e1e2;
        let _5 = this.e1e3;
        let _6 = this.e2e3;
        let _7 = this.e0e1e2e3;
        let _8 = point.x;
        let _9 = point.y;
        let _10 = point.z;

        float3 result;
        result.x = (_8) + (-2.0 * _2 * _4) + (-2.0 * _3 * _5) + (2.0 * _0 * _1) + (2.0 * _6 * _7) + (-2.0 * _0 * _10 * _5) + (-2.0 * _0 * _4 * _9) + (-2.0 * _4 * _4 * _8) + (-2.0 * _5 * _5 * _8) + (-2.0 * _5 * _6 * _9) + (2.0 * _10 * _4 * _6);
        result.y = (_9) + (-2.0 * _3 * _6) + (-2.0 * _5 * _7) + (2.0 * _0 * _2) + (2.0 * _1 * _4) + (-2.0 * _0 * _10 * _6) + (-2.0 * _10 * _4 * _5) + (-2.0 * _4 * _4 * _9) + (-2.0 * _5 * _6 * _8) + (-2.0 * _6 * _6 * _9) + (2.0 * _0 * _4 * _8);
        result.z = (_10) + (2.0 * _0 * _3) + (2.0 * _1 * _5) + (2.0 * _2 * _6) + (2.0 * _4 * _7) + (-2.0 * _10 * _5 * _5) + (-2.0 * _10 * _6 * _6) + (-2.0 * _4 * _5 * _9) + (2.0 * _0 * _5 * _8) + (2.0 * _0 * _6 * _9) + (2.0 * _4 * _6 * _8);
        return result;
    }

    Rotor rotor_part()
    {
        Rotor result;
        result.s = this.s;
        result.e1e2 = this.e1e2;
        result.e1e3 = this.e1e3;
        result.e2e3 = this.e2e3;
        return result;
    }
}
