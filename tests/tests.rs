#[macro_use]
extern crate schemamama;

use schemamama::{Adapter, Migration, Migrator, Version};
use std::cell::RefCell;
use std::collections::BTreeSet;

struct DummyAdapter {
    versions: RefCell<BTreeSet<Version>>
}

impl DummyAdapter {
    pub fn new() -> DummyAdapter {
        DummyAdapter { versions: RefCell::new(BTreeSet::new()) }
    }

    pub fn is_migrated(&self, version: Version) -> bool {
        self.versions.borrow().contains(&version)
    }
}

impl Adapter for DummyAdapter {
    type MigrationType = Migration;

    fn current_version(&self) -> Option<Version> {
        self.versions.borrow().iter().last().map(|v| *v)
    }

    fn migrated_versions(&self) -> BTreeSet<Version> {
        self.versions.borrow().iter().cloned().collect()
    }

    fn apply_migration(&self, migration: &Migration) {
        self.versions.borrow_mut().insert(migration.version());
    }

    fn revert_migration(&self, migration: &Migration) {
        self.versions.borrow_mut().remove(&migration.version());
    }
}

struct FirstMigration;
migration!(FirstMigration, 10, "first migration");
struct SecondMigration;
migration!(SecondMigration, 20, "second migration");

#[test]
fn test_registration() {
    let mut migrator = Migrator::new(DummyAdapter::new());
    assert_eq!(migrator.first_version(), None);
    assert_eq!(migrator.last_version(), None);
    migrator.register(Box::new(SecondMigration));
    migrator.register(Box::new(FirstMigration));
    assert_eq!(migrator.first_version(), Some(10));
    assert_eq!(migrator.last_version(), Some(20));
}

#[test]
fn test_version_registered() {
    let mut migrator = Migrator::new(DummyAdapter::new());
    assert_eq!(migrator.version_registered(10), false);
    migrator.register(Box::new(FirstMigration));
    assert_eq!(migrator.version_registered(10), true);
}

#[test]
fn test_migrate() {
    let mut migrator = Migrator::new(DummyAdapter::new());
    migrator.register(Box::new(FirstMigration));
    migrator.register(Box::new(SecondMigration));
    assert_eq!(migrator.current_version(), None);
    migrator.up(20);
    assert_eq!(migrator.current_version(), Some(20));
    migrator.down(Some(10));
    assert_eq!(migrator.current_version(), Some(10));
    migrator.down(None);
    assert_eq!(migrator.current_version(), None);
}

#[test]
fn test_retroactive_migrations() {
    let mut migrator = Migrator::new(DummyAdapter::new());
    migrator.register(Box::new(SecondMigration));
    migrator.up(20);
    assert_eq!(migrator.current_version(), Some(20));
    assert!(migrator.adapter().is_migrated(20));
    assert!(!migrator.adapter().is_migrated(10));
    migrator.register(Box::new(FirstMigration));
    migrator.up(20);
    assert_eq!(migrator.current_version(), Some(20));
    assert!(migrator.adapter().is_migrated(20));
    assert!(migrator.adapter().is_migrated(10));
}
