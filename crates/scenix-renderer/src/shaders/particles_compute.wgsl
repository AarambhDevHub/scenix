// Particle compute simulation (scenix v1.7.0, optional).
//
// A 64-byte pool record drives the GPU particle simulation:
//
//   pos_age   = (x, y, z, age)
//   vel_life  = (vx, vy, vz, lifetime)
//   color     = (r, g, b, a)
//   misc      = (size, rotation, pad, pad)
//
// `cs_main` integrates gravity + drag, advances age, and packs the draw
// instance record (the same 64-byte stride the billboard path consumes).
// `cs_respawn` runs as a second, sequential dispatch: one invocation scans
// from a ring cursor and respawns up to `spawn_budget` dead slots, giving
// deterministic ring-window respawn without atomics.

struct PoolItem {
    // x, y, z, age
    pos_age: vec4<f32>,
    // vx, vy, vz, lifetime
    vel_life: vec4<f32>,
    color: vec4<f32>,
    // x = size, y = rotation
    misc: vec4<f32>,
}

struct DrawInstance {
    // (position.xyz, size) — byte-identical to the billboard vertex layout.
    center_size: vec4<f32>,
    color: vec4<f32>,
    uv_rect: vec4<f32>,
    rotation_pad: vec4<f32>,
}

// Uniform layout, 128 bytes:
//   gravity_dt = (gx, gy, gz, dt)
//   color_from = birth rgba
//   color_to   = death rgba
//   knobs      = (drag, spawn_budget, hash_seed, size_from)
//   fade       = (size_to, life_min, life_max, speed_from)
//   uv_rect    = cell rect for spawned/dying particle instances
//   origin     = (spawn position, speed_to)
//   direction  = normalized spawn direction, w unused
struct SimParams {
    gravity_dt: vec4<f32>,
    color_from: vec4<f32>,
    color_to: vec4<f32>,
    knobs: vec4<f32>,
    fade: vec4<f32>,
    uv_rect: vec4<f32>,
    origin: vec4<f32>,
    direction: vec4<f32>,
}

@group(0) @binding(0)
var<storage, read_write> pool: array<PoolItem>;
@group(0) @binding(1)
var<uniform> sim: SimParams;
@group(0) @binding(2)
var<storage, read_write> ring_cursor: array<u32, 1>;
@group(0) @binding(3)
var<storage, read_write> instances: array<DrawInstance>;

// Slot-born pseudo-random in [0, 1).
fn random01(slot: u32, stream: u32) -> f32 {
    let mixed = hash32(slot * 0x2f6a335bu) + stream;
    return f32(hash32(mixed) & 0xffffffu) / 16777215.0;
}

fn hash32(input: u32) -> u32 {
    var value = input;
    value ^= value >> 16u;
    value *= 0x7feb352du;
    value ^= value >> 15u;
    value *= 0x846ca68bu;
    value ^= value >> 16u;
    return value;
}

@compute
@workgroup_size(64)
fn cs_main(
    @builtin(global_invocation_id) gid: vec3<u32>,
) {
    let pool_len = arrayLength(&pool);
    let slot = gid.x;
    if slot >= pool_len {
        return;
    }
    let instance_len = arrayLength(&instances);
    let dt = sim.gravity_dt.w;

    let particle = pool[slot];
    var out = DrawInstance();
    out.uv_rect = sim.uv_rect;
    out.rotation_pad = vec4<f32>(particle.misc.y, 0.0, 0.0, 0.0);

    if particle.pos_age.w < particle.vel_life.w {
        var integrated = particle;
        var velocity = integrated.vel_life.xyz;
        let drag = max(sim.knobs.x, 0.0);
        velocity *= max(1.0 - drag * dt, 0.0);
        velocity += sim.gravity_dt.xyz * dt;
        let stepped = integrated.pos_age.xyz + velocity * dt;
        integrated.pos_age = vec4<f32>(stepped, integrated.pos_age.w + dt);
        integrated.vel_life = vec4<f32>(velocity, integrated.vel_life.w);

        let life = clamp(integrated.pos_age.w / integrated.vel_life.w, 0.0, 1.0);
        integrated.misc.x = mix(sim.knobs.w, sim.fade.x, life);
        integrated.color = mix(sim.color_from, sim.color_to, life);

        pool[slot] = integrated;
        out.center_size = vec4<f32>(integrated.pos_age.xyz, max(integrated.misc.x, 0.0));
        out.color = integrated.color;
    } else {
        // Dead slot: park the instance with zero size until respawn hits.
        out.center_size = vec4<f32>(particle.pos_age.xyz, 0.0);
        out.color = vec4<f32>(0.0);
    }

    if slot < instance_len {
        instances[slot] = out;
    }
}

// Second dispatch: a single invocation ring-scans for dead slots and
// respawns up to `spawn_budget` of them following the ring cursor.
@compute
@workgroup_size(1)
fn cs_respawn(@builtin(global_invocation_id) gid: vec3<u32>) {
    let pool_len = arrayLength(&pool);
    if gid.x != 0u || pool_len == 0u {
        return;
    }
    let budget = u32(sim.knobs.y);
    let cursor = ring_cursor[0] % pool_len;
    var claimed = 0u;
    var scan = 0u;
    while scan < pool_len && claimed < budget {
        let slot = (cursor + scan) % pool_len;
        let particle = pool[slot];
        if particle.pos_age.w >= particle.vel_life.w {
            var born = PoolItem();
            let lifetime = mix(sim.fade.y, sim.fade.z, random01(slot, 1u));
            let speed = mix(sim.fade.w, sim.origin.w, random01(slot, 2u));
            born.pos_age = vec4<f32>(sim.origin.xyz, 0.0);
            born.vel_life = vec4<f32>(sim.direction.xyz * max(speed, 0.0), max(lifetime, 0.0));
            born.color = sim.color_from;
            born.misc = vec4<f32>(max(sim.knobs.w, 0.0), 0.0, 0.0, 0.0);
            pool[slot] = born;

            var out = DrawInstance();
            out.center_size = vec4<f32>(born.pos_age.xyz, born.misc.x);
            out.color = born.color;
            out.uv_rect = sim.uv_rect;
            out.rotation_pad = vec4<f32>(0.0, 0.0, 0.0, 0.0);
            if slot < arrayLength(&instances) {
                instances[slot] = out;
            }

            claimed += 1u;
        }
        scan += 1u;
    }
    ring_cursor[0] = (cursor + scan) % max(pool_len, 1u);
}
