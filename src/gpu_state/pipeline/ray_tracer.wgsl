@group(0) @binding(0) var colorBuffer: texture_storage_2d<rgba8unorm, write>;

@group(1) @binding(0) var<uniform> camera: Camera;
@group(1) @binding(1) var<storage, read> objects: Geometries;

@group(2) @binding(0) var randomState: texture_storage_2d<r32uint, read_write>;

struct Material {
    color: vec3<f32>,
    kind: u32,
    fuzz: f32,
}

struct Geometry {
    center: vec3<f32>,
    radius: f32,
    u: vec3<f32>,
    v: vec3<f32>,
    kind: u32,
    material: Material,
}

struct Geometries {
	geometries: array<Geometry>,
}

struct Ray {
    direction: vec3<f32>,
    origin: vec3<f32>,
}

struct Camera {
    position: vec3<f32>,
    focusDist: f32,
	forwards: vec3<f32>,
    defocusAngle: f32,
	right: vec3<f32>,
	up: vec3<f32>,
}

struct RenderState {
	t: f32,
	material: Material,
	hit: bool,
	position: vec3<f32>,
	normal: vec3<f32>,
    frontFace: bool,
}

fn random(screenPos: vec2<i32>) -> f32 {
	var x: u32 = textureLoad(randomState, screenPos).x;
	x ^= x << 13u;
	x ^= x >> 17u;
	x ^= x << 5u;
    textureStore(randomState, screenPos, vec4<u32>(x, 0u, 0u, 1u));
	return f32(x) / 4294967295.0;
}

@compute @workgroup_size(1,1,1)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    let screenSize: vec2<u32> = textureDimensions(colorBuffer);
    let screenPos: vec2<i32> = vec2<i32>(id.xy);

    let horizontalCoefficient: f32 = (f32(screenPos.x) - f32(screenSize.x) / 2.0) / f32(screenSize.x);
    let verticalCoefficient: f32 = -(f32(screenPos.y) - f32(screenSize.y) / 2.0) / f32(screenSize.x);
    let forwards: vec3<f32> = camera.forwards;
    let right: vec3<f32> = camera.right;
    let up: vec3<f32> = camera.up;

    var pixelColor = vec3<f32>(0.0, 0.0, 0.0);
    
    var myRay1: Ray;
    myRay1.origin = camera.position;
    myRay1.direction = forwards + horizontalCoefficient*right + verticalCoefficient*up;
    
    let samples: u32 = 20u;
    for(var sample: u32 = 0u; sample < samples; sample++) {
        pixelColor += rayColor(myRay1, screenPos);
    }
    
    textureStore(colorBuffer, screenPos, vec4<f32>(pixelColor/f32(samples), 1.0));
}

fn rayColor(ray: Ray, screenPos: vec2<i32>) -> vec3<f32> {

    var color: vec3<f32> = vec3(1.0, 1.0, 1.0);
    var result: RenderState;

    var temp_ray: Ray;
    temp_ray.origin = ray.origin;
    temp_ray.direction = ray.direction;

    let bounces: u32 = 5u;
    for(var bounce: u32 = 0u; bounce < bounces; bounce++) {

        result = trace(temp_ray);

        // early exit
        if (!result.hit) {
            color = color * skyColor(temp_ray);
            break;
        }

        // Set up for next trace
        temp_ray.origin = result.position;

        switch result.material.kind {
            case 0u, default {
                temp_ray.direction = lambertian_ray_direction(result, screenPos);
            }
            case 1u: {
                temp_ray.direction = metallic_ray_direction(
                    temp_ray.direction,
                    result.normal,
                    result.material.fuzz,
                    screenPos
                );
            }
        }

        // unpack color
        color = color * result.material.color;
    }

    // Rays which reached terminal state and bounced indefinitely
    if (result.hit) {
        color = vec3(0.0, 0.0, 0.0);
    }

    return color;
}

fn lambertian_ray_direction(hitRecord: RenderState, screenPos: vec2<i32>) -> vec3<f32> {
    let d = random_unit_vector(screenPos) + hitRecord.normal;
    
    // if (length(d) < 0.0001) {
    //     return hitRecord.normal;
    // }

    return normalize(d);
}

fn metallic_ray_direction(rayDirection: vec3<f32>, normal: vec3<f32>, fuzz: f32, screenPos: vec2<i32>) -> vec3<f32> {
    loop {
        let d = reflect(rayDirection, normal) + fuzz*random_unit_vector(screenPos);

        if (dot(d, normal) > 0.0) {
            return normalize(d);
        }
    }

    return vec3<f32>(0.0, 0.0, 0.0);
}

fn random_unit_vector(screenPos: vec2<i32>) -> vec3<f32> {
    let r1 = random(screenPos);
    let r2 = random(screenPos);
    let pi = radians(180.0);
    let x = cos(2.0*pi*r1)*2.0*sqrt(r2*(1.0-r2));
    let y = sin(2.0*pi*r1)*2.0*sqrt(r2*(1.0-r2));
    let z = 1.0 - 2.0*r2;
    return vec3<f32>(x,y,z);
}

fn trace(ray: Ray) -> RenderState {
    var renderState: RenderState;
    var nearestHit: f32 = 9999.0;
    
	for (var i: u32 = 0u; i < arrayLength(&objects.geometries); i++) {
        var newRenderState: RenderState;

        switch objects.geometries[i].kind {
            case 0u, default {
                newRenderState = hit_sphere(ray, objects.geometries[i], 0.001, nearestHit);        
            }
            case 1u {
                newRenderState = hit_quad(ray, objects.geometries[i], 0.001, nearestHit);
            }
        } 
        
        if (newRenderState.hit) {
            nearestHit = newRenderState.t;
            renderState = newRenderState;
        }
    }

    return renderState;
}

fn hit_sphere(ray: Ray, sphere: Geometry, tMin: f32, tMax: f32) -> RenderState {
    let co: vec3<f32> = ray.origin - sphere.center;
    let a: f32 = dot(ray.direction, ray.direction);
    let b: f32 = 2.0 * dot(ray.direction, co);
    let c: f32 = dot(co, co) - sphere.radius * sphere.radius;
    let discriminant: f32 = b * b - 4.0 * a * c;

    var renderState: RenderState;

    if (discriminant > 0.0) {

        var t: f32 = (-b - sqrt(discriminant)) / (2.0 * a);

        if t < tMin {
            t = (-b + sqrt(discriminant)) / (2.0 * a);
        }

        if (t > tMin && t < tMax) {
			renderState.position = ray.origin + t*ray.direction;
			renderState.normal = normalize(renderState.position - sphere.center);
            renderState.t = t;
            renderState.material = sphere.material;
            renderState.hit = true;
            renderState.frontFace = dot(ray.direction, renderState.normal) < 0.0;
            if !renderState.frontFace {
                renderState.normal = -renderState.normal;
            }
            return renderState;
        }
    }

    renderState.hit = false;
    return renderState;
}

fn hit_quad(ray: Ray, quad: Geometry, tMin: f32, tMax: f32) -> RenderState {
    var renderState: RenderState;
    renderState.hit = false;

    var n = cross(quad.u, quad.v);
    let w = n/dot(n,n);
    n = normalize(n);
    let D = dot(n, quad.center);

    if abs(dot(n, ray.direction)) < 0.0001 {
        return renderState;
    }

    let t = (D - dot(n, ray.origin))/dot(n, ray.direction);
    let p = ray.origin + t*ray.direction - quad.center;
    let alpha = dot(w, cross(p, quad.v));
    let beta = dot(w, cross(quad.u, p));

    if (alpha > 0.0 && alpha < 1.0 && beta > 0.0 && beta < 1.0 && t > tMin && t < tMax) {
        renderState.position = ray.origin + t*ray.direction;
		renderState.normal = n;
        renderState.t = t;
        renderState.material = quad.material;
        renderState.hit = true;
        renderState.frontFace = dot(ray.direction, renderState.normal) < 0.0;
        if !renderState.frontFace {
            renderState.normal = -renderState.normal;
        }
    }

    return renderState;
}

fn skyColor(ray: Ray) -> vec3<f32> {
    let unit_direction = normalize(ray.direction);
    let a = 0.5*(unit_direction.z + 1.0);
    let color1 = vec3<f32>(0.5, 0.7, 1.0);
    let color2 = vec3<f32>(1.0, 1.0, 1.0);
    return (1.0 - a)*color2 + a*color1;
}
