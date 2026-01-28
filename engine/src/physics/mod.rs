use rapier3d::prelude::*;

pub struct PhysicsWorld {
    pipeline: PhysicsPipeline,
    gravity: Vector<Real>,
    integration_parameters: IntegrationParameters,
    islands: IslandManager,
    broad_phase: BroadPhase,
    narrow_phase: NarrowPhase,
    bodies: RigidBodySet,
    colliders: ColliderSet,
    ccd_solver: CCDSolver,
}

impl PhysicsWorld {
    pub fn new() -> Self {
        Self {
            pipeline: PhysicsPipeline::new(),
            gravity: vector![0.0, -9.81, 0.0],
            integration_parameters: IntegrationParameters::default(),
            islands: IslandManager::new(),
            broad_phase: BroadPhase::new(),
            narrow_phase: NarrowPhase::new(),
            bodies: RigidBodySet::new(),
            colliders: ColliderSet::new(),
            ccd_solver: CCDSolver::new(),
        }
    }

    pub fn step(&mut self, delta_seconds: f32) {
        self.integration_parameters.dt = delta_seconds;
        let mut query_pipeline = QueryPipeline::new();
        let hooks = ();
        let events = ();
        self.pipeline.step(
            &self.gravity,
            &self.integration_parameters,
            &mut self.islands,
            &mut self.broad_phase,
            &mut self.narrow_phase,
            &mut self.bodies,
            &mut self.colliders,
            &mut self.ccd_solver,
            &(),
            &hooks,
            &events,
        );
        query_pipeline.update(&self.bodies, &self.colliders);
    }
}
