use std::collections::{BTreeMap, BTreeSet};

use derive_more::derive::{Constructor, Deref, DerefMut};

use bevy::prelude::{Component, Name, Reflect};

use crate::{Aspect, Augmentation, Element};

#[derive(Default)]
#[derive(Component)]
#[require(Health(|| Health::PLAYER_MAX), PlayerElements(PlayerElements::rock_paper_scissors), Name(|| Name::new("Player")))]
pub struct Player;

#[derive(Clone)]
#[derive(Deref, DerefMut)]
#[derive(Component)]
pub struct Health(u32);

impl Health {
    pub const PLAYER_MAX: Self = Health(25);
    pub const ENEMY_MAX: Self = Health(5);

    pub fn new(hp: u32) -> Self {
        Health(hp)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
#[derive(Constructor)]
pub struct ElementUpgrades {
    pub augmentation: Option<Augmentation>,
    pub aspect: Option<Aspect>,
}

#[derive(Clone, Debug, Default)]
#[derive(Constructor)]
#[derive(Component, Reflect)]
pub struct PlayerElements {
    pub elements: BTreeSet<Element>,
    pub augmentations: BTreeMap<Element, Augmentation>,
    pub enchantments: BTreeMap<Element, Aspect>,
}

impl PlayerElements {
    pub fn from_set(elements: impl IntoIterator<Item = Element>) -> Self {
        Self {
            elements: elements.into_iter().collect(),
            augmentations: BTreeMap::default(),
            enchantments: BTreeMap::default(),
        }
    }

    pub fn insert(&mut self, element: Element) {
        self.elements.insert(element);
    }

    pub fn augment(&mut self, element: Element, augmentation: Augmentation) {
        assert!(self.elements.contains(&element));
        self.augmentations.insert(element, augmentation);
    }

    pub fn enchant(&mut self, element: Element, aspect: Aspect) {
        assert!(self.elements.contains(&element));
        self.enchantments.insert(element, aspect);
    }

    pub fn get_enchantment(&self, element: Element) -> Option<&Aspect> {
        if !self.elements.contains(&element) {
            bevy::log::warn!(
                "Augmentation requested for an element the player doesn't have: {element} {:?}",
                self.elements
            );
        }
        self.enchantments.get(&element)
    }

    pub fn get_augmentation(&self, element: Element) -> Option<&Augmentation> {
        if !self.elements.contains(&element) {
            bevy::log::warn!(
                "Augmentation requested for an element the player doesn't have: {element} {:?}",
                self.elements
            );
        }
        self.augmentations.get(&element)
    }

    fn rock_paper_scissors() -> Self {
        Self::from_set([Element::Rock, Element::Paper, Element::Scissors])
    }
}
