// engine_core/src/ecs/has_any.rs
use crate::ecs::{entity::Entity, ecs::Ecs};

/// Trait that can test if an entity has any given component types.
pub trait HasAny {
    /// Returns `true` if the entity has at least one of the supplied component types.
    fn has_any(world_ecs: &Ecs, entity: Entity) -> bool;
}

macro_rules! impl_has_any_for_tuples {
    // Empty tuple
    () => {
        impl HasAny for () {
            #[inline]
            fn has_any(_world_ecs: &Ecs, _entity: Entity) -> bool { false }
        }
    };

    // Single element tuple
    ( $head:ident ) => {
        impl<$head> HasAny for ($head,)
        where
            $head: crate::ecs::component::Component + 'static,
        {
            #[inline]
            fn has_any(world_ecs: &Ecs, entity: Entity) -> bool {
                world_ecs.has::<$head>(entity)
            }
        }
    };

    // More than two elements
    ( $head:ident, $( $tail:ident ),+ ) => {
        impl<$head, $( $tail ),+> HasAny for ($head, $( $tail ),+)
        where
            $head: crate::ecs::component::Component + 'static,
            $( $tail: crate::ecs::component::Component + 'static, )+
        {
            #[inline]
            fn has_any(world_ecs: &Ecs, entity: Entity) -> bool {
                if world_ecs.has::<$head>(entity) {
                    true
                } else {
                    <( $( $tail, )+ ) as HasAny>::has_any(world_ecs, entity)
                }
            }
        }

        // Recursion step
        impl_has_any_for_tuples!( $( $tail ),+ );
    };
}

// Generate implementations for tuples up to length E (Add more letters to extend).
impl_has_any_for_tuples!(A, B, C, D, E);