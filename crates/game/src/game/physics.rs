use bevy::prelude::*;
use physx::prelude::*;

pub struct PhysicsPlugin;
impl Plugin for PhysicsPlugin {
    fn build(&self, app: &mut App) {
        // Add the physics system
        app.add_systems(Startup, init);
    }
}

type PxMaterial = physx::material::PxMaterial<()>;
type PxShape = physx::shape::PxShape<(), PxMaterial>;
type PxArticulationLink = physx::articulation_link::PxArticulationLink<(), PxShape>;
type PxRigidStatic = physx::rigid_static::PxRigidStatic<(), PxShape>;
type PxRigidDynamic = physx::rigid_dynamic::PxRigidDynamic<(), PxShape>;
type PxArticulationReducedCoordinate =
    physx::articulation_reduced_coordinate::PxArticulationReducedCoordinate<(), PxArticulationLink>;
type PxScene = physx::scene::PxScene<
    *const std::ffi::c_void,
    PxArticulationLink,
    PxRigidStatic,
    PxRigidDynamic,
    PxArticulationReducedCoordinate,
    OnCollision,
    OnTrigger,
    OnConstraintBreak,
    OnWakeSleep,
    OnAdvance,
>;

/// Next up, the simulation event callbacks need to be defined, and possibly an
/// allocator callback as well.
struct OnCollision;
impl CollisionCallback for OnCollision {
    fn on_collision(
        &mut self,
        _header: &physx_sys::PxContactPairHeader,
        _pairs: &[physx_sys::PxContactPair],
    ) {
    }
}
struct OnTrigger;
impl TriggerCallback for OnTrigger {
    fn on_trigger(&mut self, _pairs: &[physx_sys::PxTriggerPair]) {}
}

struct OnConstraintBreak;
impl ConstraintBreakCallback for OnConstraintBreak {
    fn on_constraint_break(&mut self, _constraints: &[physx_sys::PxConstraintInfo]) {}
}
struct OnWakeSleep;
impl WakeSleepCallback<PxArticulationLink, PxRigidStatic, PxRigidDynamic> for OnWakeSleep {
    fn on_wake_sleep(
        &mut self,
        _actors: &[&physx::actor::ActorMap<PxArticulationLink, PxRigidStatic, PxRigidDynamic>],
        _is_waking: bool,
    ) {
    }
}

struct OnAdvance;
impl AdvanceCallback<PxArticulationLink, PxRigidDynamic> for OnAdvance {
    fn on_advance(
        &self,
        _actors: &[&physx::rigid_body::RigidBodyMap<PxArticulationLink, PxRigidDynamic>],
        _transforms: &[PxTransform],
    ) {
    }
}

fn init() {
    // Holds a PxFoundation and a PxPhysics.
    // Also has an optional Pvd and transport, not enabled by default.
    // The default allocator is the one provided by PhysX.
    let mut physics = PhysicsFoundation::<_, PxShape>::default();

    // Setup the scene object.  The PxScene type alias makes this much cleaner.
    // There are lots of unwrap calls due to potential null pointers.
    let mut scene: Owner<PxScene> = physics
        .create(SceneDescriptor {
            gravity: PxVec3::new(0.0, -9.81, 0.0),
            on_advance: Some(OnAdvance),
            ..SceneDescriptor::new(std::ptr::null())
        })
        .unwrap();

    let mut material = physics.create_material(0.5, 0.5, 0.6, ()).unwrap();

    let ground_plane = physics
        .create_plane(PxVec3::new(0.0, 1.0, 0.0), 0.0, material.as_mut(), ())
        .unwrap();
    // The scene owns actors that are added to it.  They can be retrieved using the
    // various getters on the scene.
    scene.add_static_actor(ground_plane);

    let sphere_geo = PxSphereGeometry::new(10.0);

    let mut sphere_actor = physics
        .create_rigid_dynamic(
            PxTransform::from_translation(&PxVec3::new(0.0, 40.0, 100.0)),
            &sphere_geo,
            material.as_mut(),
            10.0,
            PxTransform::default(),
            (),
        )
        .unwrap();
    sphere_actor.set_angular_damping(0.5);
    sphere_actor.set_rigid_body_flag(RigidBodyFlag::EnablePoseIntegrationPreview, true);
    scene.add_dynamic_actor(sphere_actor);

    // SAFETY: scratch buffer creation
    #[allow(unsafe_code)]
    let mut scratch = unsafe { ScratchBuffer::new(4) };

    // Updating
    let heights_over_time = (0..100)
        .map(|_| {
            // Calls both simulate and fetch_results.  More complex simulation update
            // controls are not fully supported.
            scene
                .step(
                    0.1,
                    None::<&mut physx_sys::PxBaseTask>,
                    Some(&mut scratch),
                    true,
                )
                .expect("error occured during simulation");
            // For simplicity, just read out the only dynamic actor in the scene.
            // getActiveActors is also supported, it returns a Vec<&mut ActorMap> which has
            // a map method that takes a function for each actor type, and `as_<T>` methods
            // that return an Option<&mut T>.
            let actors = scene.get_dynamic_actors();
            actors[0].get_global_position().y() as i32 - 10
        })
        .collect::<Vec<_>>();

    // Draw to stdout
    let max_h = 18;
    (0..max_h)
        .map(|h| {
            let h = max_h - 1 - h;
            heights_over_time
                .iter()
                .map(|p| if h == *p { 'o' } else { ' ' })
                .collect::<String>()
        })
        .for_each(|line| println!("{}", line));

    println!("Simulation complete");
}