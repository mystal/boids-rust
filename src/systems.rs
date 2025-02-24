use crate::components::{Position, Velocity};
use crate::{
    Boid, DeltaTime, Renderable, COHERENCE_FACTOR, HEIGHT, MAX_PROXIMAL_BOIDS, MAX_SPEED, SCALE,
    SEPARATION_FACTOR, WIDTH,
};
use rltk::Rltk;
use specs::prelude::*;

pub struct MovementSys;
impl<'a> System<'a> for MovementSys {
    type SystemData = (
        Read<'a, DeltaTime>,
        WriteStorage<'a, Position>,
        ReadStorage<'a, Velocity>,
    );

    fn run(&mut self, (delta, mut pos, vel): Self::SystemData) {
        let delta = delta.0;
        for (pos, vel) in (&mut pos, &vel).join() {
            update_position(pos, vel, delta);
        }
    }
}

fn update_position(pos: &mut Position, vel: &Velocity, delta: f32) {
    pos.x += vel.x as f64 * delta as f64;
    pos.y += vel.y as f64 * delta as f64;
}

pub struct BoidSystem<'a> {
    pub ctx: &'a mut Rltk,
}
impl<'a> System<'a> for BoidSystem<'_> {
    #[allow(clippy::type_complexity)]
    type SystemData = (
        WriteStorage<'a, Position>,
        ReadStorage<'a, Renderable>,
        ReadStorage<'a, Boid>,
        WriteStorage<'a, Velocity>,
    );

    fn run(&mut self, (mut positions, renders, boids, mut velocities): Self::SystemData) {
        let mut all_positions = Vec::<Position>::new();
        let mut pos_vel_map = std::collections::HashMap::<Position, Velocity>::new();

        for (pos, vel) in (&positions, &velocities).join() {
            all_positions.push(pos.clone());
            pos_vel_map.insert(pos.clone(), vel.clone());
        }

        for (pos, render, _, vel) in (&mut positions, &renders, &boids, &mut velocities).join()
        {
            self.draw_boid(pos, render);
            if pos.x > WIDTH {
                pos.x = 0.0;
            }
            if pos.x < 0.0 {
                pos.x = WIDTH;
            }
            if pos.y > HEIGHT {
                pos.y = 0.0;
            }
            if pos.y < 0.0 {
                pos.y = HEIGHT;
            }

            self.neighbours(pos, &mut all_positions);
            self.separate(pos, vel, &all_positions);
            self.align(pos, vel, &all_positions, &pos_vel_map);
            self.cohere(pos, vel, &all_positions);
            self.limit_speed(vel);
        }
    }
}

impl<'a> BoidSystem<'a> {
    pub fn draw_boid(&mut self, pos: &Position, render: &Renderable) {
        self.ctx.set(
            pos.x as i32,
            pos.y as i32,
            render.fg,
            render.bg,
            rltk::to_cp437('▲'),
        );
    }

    pub fn limit_speed(&self, vel: &mut Velocity) {
        if vel.x > MAX_SPEED {
            vel.x = MAX_SPEED;
        }
        if vel.x < -MAX_SPEED {
            vel.x = -MAX_SPEED;
        }
        if vel.y > MAX_SPEED {
            vel.y = MAX_SPEED;
        }
        if vel.y < -MAX_SPEED {
            vel.y = -MAX_SPEED;
        }
    }

    pub fn neighbours(&self, pos: &Position, positions: &mut Vec<Position>) {
        positions.sort_unstable_by(|a, b| pos.distance(a, b));
    }

    pub fn separate(&self, pos: &mut Position, vel: &mut Velocity, positions: &[Position]) {
        let (mut x, mut y) = (0.0, 0.0);

        for i in 0..MAX_PROXIMAL_BOIDS {
            let other_pos = &positions[i as usize];
            if pos.distance_to(other_pos) < SEPARATION_FACTOR {
                x += pos.x - other_pos.x;
                y += pos.y - other_pos.y;
            }
        }
        vel.x = x * SCALE;
        vel.y = y * SCALE;
        pos.x += x;
        pos.y += y;
    }

    pub fn align(
        &self,
        pos: &mut Position,
        vel: &mut Velocity,
        positions: &[Position],
        map: &std::collections::HashMap<Position, Velocity>,
    ) {
        let (mut x, mut y) = (0.0 as f64, 0.0 as f64);

        for i in 0..MAX_PROXIMAL_BOIDS {
            let other_pos = &positions[i as usize];
            match map.get(other_pos) {
                None => continue,
                Some(vel) => {
                    x += vel.x; // I need here the other velocity
                    y += vel.y; // I need here the other velocity
                }
            }
        }

        let (dx, dy) = (x / MAX_PROXIMAL_BOIDS as f64, y / MAX_PROXIMAL_BOIDS as f64);
        vel.x += dx * SCALE;
        vel.y += dy * SCALE;
        pos.x += dx;
        pos.y += dy;
    }

    pub fn cohere(&self, pos: &mut Position, vel: &mut Velocity, positions: &[Position]) {
        let (mut x, mut y) = (0.0 as f64, 0.0 as f64);

        for i in 0..MAX_PROXIMAL_BOIDS {
            let other_pos = &positions[i as usize];
            x += other_pos.x;
            y += other_pos.y;
        }

        let (dx, dy) = (
            ((x / MAX_PROXIMAL_BOIDS as f64) - pos.x) / COHERENCE_FACTOR,
            ((y / MAX_PROXIMAL_BOIDS as f64) - pos.y) / COHERENCE_FACTOR,
        );
        vel.x += dx * SCALE;
        vel.y += dy * SCALE;
        pos.x += dx;
        pos.y += dy;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn text_sort_stable() {
        let pos = Position::new(20.0, 20.0);
        let mut positions = vec![
            Position::new(25.0, 25.0),
            Position::new(10.0, 10.0),
            Position::new(15.0, 15.0),
            pos.clone(),
        ];
        positions.sort_unstable_by(|a, b| pos.distance(a, b));
        assert_eq!(
            vec![
                Position::new(20.0, 20.0),
                Position::new(25.0, 25.0),
                Position::new(15.0, 15.0),
                Position::new(10.0, 10.0),
            ],
            positions
        );
    }
}
