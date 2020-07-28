#version 460

#include "../per_frame_resources.glsl"
#include "../utilities.glsl"

layout(location = 0) in vec3 in_WorldPosition;
layout(location = 1) in vec3 in_ParticleWorldPosition;
layout(location = 2) in float in_Radius;
layout(location = 0) out float out_Depth;
layout(location = 1) out float out_Thickness;

// Note that we promise to only lessen the depth value, so gpu can still do some hi-z/early depth culling
layout(depth_less) out float gl_FragDepth;

void main() {
    vec3 rayDir = normalize(in_WorldPosition - Camera.Position);
    float cameraDistance;
    float cameraDistanceFar;
    // TODO: Elipsoids using APIC matrix?
    if (!sphereIntersect(in_ParticleWorldPosition, in_Radius, Camera.Position, rayDir, cameraDistance, cameraDistanceFar))
        discard;

    vec3 cameraPosToSpherePos = cameraDistance * rayDir;
    vec3 sphereWorldPos = Camera.Position + cameraPosToSpherePos;
    vec3 normal = (sphereWorldPos - in_ParticleWorldPosition) / in_Radius;

    // Adjust depth buffer value.
    // TODO: Necessary for this step?
    vec2 projected_zw = (Camera.ViewProjection * vec4(sphereWorldPos, 1.0)).zw; // (trusting optimizer to pick the right thing ;-))
    gl_FragDepth = projected_zw.x / projected_zw.y;

    // There's more clever ways to store depth for this particular purpose here, but going with the standard one is simply less confusing.
    out_Depth = gl_FragDepth;
    // quadratic splats
    out_Thickness = sq((cameraDistanceFar - cameraDistance) / in_Radius) * in_Radius;
}
