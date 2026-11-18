//! Posing a character touches only its own body's and head's nodes, so a
//! character posed beside others never reads what another one writes.

use bevy::prelude::*;

pub(crate) trait Transforms {
    fn read(&self, entity: Entity) -> Option<&Transform>;
    fn write(&mut self, entity: Entity) -> Option<&mut Transform>;
    /// A transform left bit for bit as it was is not written, so it is not
    /// marked changed.
    fn set(&mut self, entity: Entity, to: impl FnOnce(&Transform) -> Transform);
}

impl Transforms for Query<'_, '_, &mut Transform> {
    fn read(&self, entity: Entity) -> Option<&Transform> {
        self.get(entity).ok()
    }

    fn write(&mut self, entity: Entity) -> Option<&mut Transform> {
        self.get_mut(entity).ok().map(Mut::into_inner)
    }

    fn set(&mut self, entity: Entity, to: impl FnOnce(&Transform) -> Transform) {
        if let Ok(mut now) = self.get_mut(entity) {
            let new = to(&now);
            if !same(&now, &new) {
                *now = new;
            }
        }
    }
}

/// Which character owns each node while the characters are posed side by
/// side, by the node's entity index. An index has at most one owner, so two
/// characters never reach the same transform.
#[derive(Default)]
pub(crate) struct Owners(Vec<u32>);

impl Owners {
    /// Gives `owner` (not 0) the nodes `nodes`. It is false if another owner
    /// already holds one of them. The nodes given before that one stay
    /// `owner`'s until `release`.
    pub(crate) fn claim(&mut self, owner: u32, nodes: &[Entity]) -> bool {
        for node in nodes {
            let index = node.index_u32() as usize;
            if index >= self.0.len() {
                self.0.resize(index + 1, 0);
            }
            match self.0[index] {
                0 => self.0[index] = owner,
                held if held == owner => {}
                _ => return false,
            }
        }
        true
    }

    /// Takes back from `owner` the nodes of `nodes` it holds.
    pub(crate) fn release(&mut self, owner: u32, nodes: &[Entity]) {
        for node in nodes {
            if let Some(held) = self.0.get_mut(node.index_u32() as usize)
                && *held == owner
            {
                *held = 0;
            }
        }
    }

    /// Whether `owner` holds `entity`.
    pub(crate) fn owns(&self, owner: u32, entity: Entity) -> bool {
        self.0.get(entity.index_u32() as usize) == Some(&owner)
    }
}

/// One character's view of the world's transforms while the characters are
/// posed side by side. It reads and writes the world directly, but only the
/// nodes its owner holds in `Owners`. Any other node reads as missing. It
/// marks changed only the transforms that end posing different, bit for bit,
/// from how they began it.
pub(crate) struct Owned<'q, 'w, 's, 'd> {
    world: &'q Query<'w, 's, &'d mut Transform>,
    owners: &'q Owners,
    owner: u32,
}

thread_local! {
    /// The owned nodes' transforms before posing (`Owned::pose`), kept so
    /// that posing allocates nothing once warm.
    static BEFORE: std::cell::Cell<Vec<Option<Transform>>> = std::cell::Cell::default();
}

impl<'q, 'w, 's, 'd> Owned<'q, 'w, 's, 'd> {
    /// Runs `pose` on `owner`'s view of `world`. `nodes` are the nodes
    /// `owner` holds in `owners`. While any `Owned` of `world` lives, nothing
    /// else may write `world`, and `owners` may not change. Holding both by
    /// shared reference ensures it.
    pub(crate) fn pose(
        (world, owners): (&'q Query<'w, 's, &'d mut Transform>, &'q Owners),
        (owner, nodes): (u32, &[Entity]),
        pose: impl FnOnce(&mut Self),
    ) {
        let mut before = BEFORE.take();
        before.clear();
        before.extend(nodes.iter().map(|&node| world.get(node).ok().copied()));
        pose(&mut Owned {
            world,
            owners,
            owner,
        });
        for (&node, before) in nodes.iter().zip(&before) {
            if let Some(before) = before
                && owners.owns(owner, node)
                // SAFETY: As in `write`.
                && let Ok(mut now) = unsafe { world.get_unchecked(node) }
                && !same(&now, before)
            {
                now.set_changed();
            }
        }
        BEFORE.set(before);
    }

    fn owns(&self, entity: Entity) -> bool {
        let owns = self.owners.owns(self.owner, entity);
        debug_assert!(owns, "a character posed a node it does not own");
        owns
    }
}

impl Transforms for Owned<'_, '_, '_, '_> {
    fn read(&self, entity: Entity) -> Option<&Transform> {
        self.owns(entity).then(|| self.world.get(entity).ok())?
    }

    fn write(&mut self, entity: Entity) -> Option<&mut Transform> {
        if !self.owns(entity) {
            return None;
        }
        // SAFETY: Only this owner reaches the entity's index (`Owners`), and
        // `&mut self` keeps this the only reference it holds to the
        // transform. Nothing else reaches the query while an `Owned` of it
        // lives (`pose`). The transform stays where it is in its table for
        // as long as the query is borrowed.
        let mut now = unsafe { self.world.get_unchecked(entity) }.ok()?;
        let now: *mut Transform = now.bypass_change_detection();
        // SAFETY: As above.
        Some(unsafe { &mut *now })
    }

    fn set(&mut self, entity: Entity, to: impl FnOnce(&Transform) -> Transform) {
        if let Some(now) = self.write(entity) {
            *now = to(now);
        }
    }
}

/// So 0.0 and -0.0 differ, and a NaN equals itself.
fn same(a: &Transform, b: &Transform) -> bool {
    bits(a) == bits(b)
}

fn bits(transform: &Transform) -> [u32; 10] {
    let [a, b, c] = transform.translation.to_array();
    let [d, e, f, g] = transform.rotation.to_array();
    let [h, i, j] = transform.scale.to_array();
    [a, b, c, d, e, f, g, h, i, j].map(f32::to_bits)
}
