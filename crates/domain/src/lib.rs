//! Core domain model: stable identifiers, revisions, and aggregate contracts.
//!
//! Depends only on the cross-cutting `contracts` crate. Deterministic
//! implementations for tests are available under the non-default `test-support`
//! feature.

#![forbid(unsafe_code)]
#![cfg_attr(coverage_nightly, feature(coverage_attribute))]

pub mod entity;
pub mod identifier;
pub mod revision;

pub use entity::Entity;
pub use identifier::EntityId;
pub use revision::Revision;

const _: Option<&dyn Entity> = None;

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use super::entity::{Entity, FreshEntity, RevisionedEntity};
    use super::identifier::EntityId;
    use super::revision::Revision;

    #[test]
    fn revision_advances() {
        assert_eq!(Revision::INITIAL.get(), 1);
        assert_eq!(Revision::INITIAL.next().expect("has successor").get(), 2);
    }

    #[test]
    fn entities_are_substitutable() {
        let id = EntityId::parse("peer:node-01").expect("valid id");
        let entities: [Box<dyn Entity>; 2] = [
            Box::new(RevisionedEntity::new(
                id.clone(),
                Revision::INITIAL.next().expect("has successor"),
            )),
            Box::new(FreshEntity::new(id.clone())),
        ];
        assert_eq!(entities[0].id(), &id);
        assert_eq!(entities[0].revision().get(), 2);
        assert_eq!(entities[1].revision(), Revision::INITIAL);
    }
}
