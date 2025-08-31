use crate::ecs::{entity::Entity, world_ecs::WorldEcs};

pub struct QueryMut<'a, C> {
    world: &'a mut WorldEcs,
    marker: std::marker::PhantomData<C>,
}

pub trait QueryTuple<'a> {
    type Iter: Iterator<Item = (Entity, Self)>;
    fn iter(world: &'a mut WorldEcs) -> Self::Iter;
}

impl<'a> QueryTuple<'a> for () {
    type Iter = std::iter::Empty<(Entity, ())>;
    fn iter(_world: &'a mut WorldEcs) -> Self::Iter {
        std::iter::empty()
    }
}

pub struct QueryIter<'a, T> {
    world: &'a mut WorldEcs,
    smallest: Vec<Entity>,
    idx: usize,
    _marker: std::marker::PhantomData<T>,
}

impl<'a, T> QueryIter<'a, T>
where
    T: QueryTuple<'a>,
{
    fn new(world: &'a mut WorldEcs) -> Self {
        // Find the store with the fewest entries.
        // The macro below expands the list of stores we have.
        let mut lens = Vec::new();

        // ---- Begin auto‑generated part (add/remove stores here) ----
        lens.push(("positions", world.positions.data.len()));
        lens.push(("velocities", world.velocities.data.len()));
        lens.push(("sprites", world.sprites.data.len()));
        // ---- End auto‑generated part ----

        // Pick the smallest non‑empty store.
        let (smallest_name, _) = lens
            .into_iter()
            .min_by_key(|(_, len)| *len)
            .unwrap_or(("positions", 0));

        let smallest = match smallest_name {
            "positions" => world.positions.data.keys().cloned().collect(),
            "velocities" => world.velocities.data.keys().cloned().collect(),
            "sprites" => world.sprites.data.keys().cloned().collect(),
            _ => Vec::new(),
        };

        QueryIter {
            world,
            smallest,
            idx: 0,
            _marker: std::marker::PhantomData,
        }
    }
}