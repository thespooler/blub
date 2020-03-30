use rand::prelude::*;

pub struct FluidWorld {
    //gravity: cgmath::Vector3<f32>, // global gravity force in m/s² (== N/kg)
    grid_dimension: cgmath::Vector3<u32>,

    particles: wgpu::Buffer,
    num_particles: u32,
}

// todo: probably want to split this up into several buffers
#[repr(C)]
#[derive(Clone, Copy)]
struct Particle {
    // Particle positions are in grid space.
    position: cgmath::Point3<f32>,
    padding: f32,
}

// #[repr(C)]
// #[derive(Clone, Copy)]
// pub struct FluidWorldUniformBufferContent {
//     pub num_particles: u32,
// }
// pub type FluidWorldUniformBuffer = UniformBuffer<FluidWorldUniformBufferContent>;

impl FluidWorld {
    // particles are distributed 2x2x2 within a single gridcell
    // (seems to be widely accepted as the default)
    const PARTICLES_PER_GRID_CELL: u32 = 8;

    pub fn new(device: &wgpu::Device, grid_dimension: cgmath::Vector3<u32>) -> Self {
        FluidWorld {
            //gravity: cgmath::Vector3::new(0.0, -9.81, 0.0), // there needs to be some grid->world relation
            grid_dimension,

            // dummy. is there an invalid buffer type?
            particles: device.create_buffer(&wgpu::BufferDescriptor {
                size: 1,
                usage: wgpu::BufferUsage::STORAGE,
            }),
            num_particles: 0,
        }
    }

    fn clamp_to_grid(&self, grid_cor: cgmath::Point3<f32>) -> cgmath::Point3<u32> {
        cgmath::Point3::new(
            self.grid_dimension.x.min(grid_cor.x as u32),
            self.grid_dimension.y.min(grid_cor.y as u32),
            self.grid_dimension.z.min(grid_cor.z as u32),
        )
    }

    // Adds a cube of fluid. Coordinates are in grid space! Very slow operation!
    // todo: Removes all previously added particles.
    pub fn add_fluid_cube(&mut self, device: &wgpu::Device, min_grid: cgmath::Point3<f32>, max_grid: cgmath::Point3<f32>) {
        // align to whole cells for simplicity.
        let min_grid = self.clamp_to_grid(min_grid);
        let max_grid = self.clamp_to_grid(max_grid);
        let extent_cell = max_grid - min_grid;

        let num_new_particles = (max_grid.x - min_grid.x) * (max_grid.y - min_grid.y) * (max_grid.z - min_grid.z) * Self::PARTICLES_PER_GRID_CELL;

        // TODO: Keep previous particles! Maybe just have a max particle num on creation and keep track of how many we actually use.
        self.num_particles = num_new_particles;
        let particle_buffer_mapping =
            device.create_buffer_mapped(self.num_particles as usize, wgpu::BufferUsage::STORAGE | wgpu::BufferUsage::STORAGE_READ);

        // Fill buffer with particle data
        let mut rng: rand::rngs::SmallRng = rand::SeedableRng::seed_from_u64(num_new_particles as u64);
        for (i, position) in particle_buffer_mapping.data.iter_mut().enumerate() {
            //let sample_idx = i as u32 % Self::PARTICLES_PER_GRID_CELL;
            let cell = cgmath::Point3::new(
                (i as u32 / Self::PARTICLES_PER_GRID_CELL % extent_cell.x) as f32,
                (i as u32 / Self::PARTICLES_PER_GRID_CELL / extent_cell.x % extent_cell.y) as f32,
                (i as u32 / Self::PARTICLES_PER_GRID_CELL / extent_cell.x / extent_cell.y) as f32,
            );
            *position = Particle {
                position: (cell + rng.gen::<cgmath::Vector3<f32>>()),
                padding: i as f32,
            };
        }

        self.particles = particle_buffer_mapping.finish();
    }

    pub fn num_particles(&self) -> u32 {
        self.num_particles
    }

    pub fn particle_buffer(&self) -> &wgpu::Buffer {
        &self.particles
    }

    pub fn particle_buffer_size(&self) -> u64 {
        self.num_particles as u64 * std::mem::size_of::<Particle>() as u64
    }

    // todo: timing
    pub fn step(&self) {}
}
