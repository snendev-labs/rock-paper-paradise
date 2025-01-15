use derive_more::derive::{
    Add, AddAssign, Constructor, Deref, DerefMut, Display, Mul, MulAssign, Sum,
};
use itertools::Itertools;
use rand::seq::{IteratorRandom, SliceRandom};
use rand::RngCore;
use std::cmp::Ordering;
use std::collections::{BTreeMap, BTreeSet};

use bevy::prelude::{Alpha, App, Color, Component, Plugin, Reflect};

pub struct RpsPlugin;

impl Plugin for RpsPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Guess>()
            .register_type::<crate::PlayerElements>()
            .register_type::<BTreeSet<Element>>()
            .register_type::<Element>();
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[derive(Display)]
#[derive(Reflect)]
pub enum Element {
    Rock,
    Water,
    Air,
    Paper,
    Earth,
    Scissors,
    Fire,
}

impl Element {
    pub const ALL: [Element; 7] = [
        Element::Rock,
        Element::Water,
        Element::Air,
        Element::Paper,
        Element::Earth,
        Element::Scissors,
        Element::Fire,
    ];

    fn naive_compare(self, rhs: Element) -> Ordering {
        match (self, rhs) {
            (Element::Rock, Element::Rock) => Ordering::Equal,
            (Element::Rock, Element::Water | Element::Air | Element::Paper) => Ordering::Less,
            (Element::Rock, Element::Earth | Element::Scissors | Element::Fire) => {
                Ordering::Greater
            }

            (Element::Water, Element::Water) => Ordering::Equal,
            (Element::Water, Element::Air | Element::Paper | Element::Earth) => Ordering::Less,
            (Element::Water, Element::Scissors | Element::Fire | Element::Rock) => {
                Ordering::Greater
            }

            (Element::Air, Element::Air) => Ordering::Equal,
            (Element::Air, Element::Paper | Element::Earth | Element::Scissors) => Ordering::Less,
            (Element::Air, Element::Fire | Element::Rock | Element::Water) => Ordering::Greater,

            (Element::Paper, Element::Paper) => Ordering::Equal,
            (Element::Paper, Element::Earth | Element::Scissors | Element::Fire) => Ordering::Less,
            (Element::Paper, Element::Rock | Element::Air | Element::Water) => Ordering::Greater,

            (Element::Earth, Element::Earth) => Ordering::Equal,
            (Element::Earth, Element::Scissors | Element::Fire | Element::Rock) => Ordering::Less,
            (Element::Earth, Element::Water | Element::Air | Element::Paper) => Ordering::Greater,

            (Element::Scissors, Element::Scissors) => Ordering::Equal,
            (Element::Scissors, Element::Fire | Element::Rock | Element::Water) => Ordering::Less,
            (Element::Scissors, Element::Air | Element::Paper | Element::Earth) => {
                Ordering::Greater
            }

            (Element::Fire, Element::Fire) => Ordering::Equal,
            (Element::Fire, Element::Rock | Element::Water | Element::Air) => Ordering::Less,
            (Element::Fire, Element::Paper | Element::Earth | Element::Scissors) => {
                Ordering::Greater
            }
        }
    }

    pub fn random(rng: &mut impl RngCore) -> Self {
        *Self::ALL.choose(rng).unwrap()
    }

    pub fn random_without(rng: &mut impl RngCore, exception: Self) -> Self {
        let mut others = BTreeSet::from(Self::ALL);
        others.remove(&exception);
        others.into_iter().choose(rng).unwrap()
    }

    pub fn random_set(rng: &mut impl RngCore, amount: usize) -> BTreeSet<Self> {
        Self::ALL.choose_multiple(rng, amount).cloned().collect()
    }

    pub fn random_item(rng: &mut impl RngCore, set: &[Self]) -> Self {
        *set.choose(rng).unwrap()
    }

    pub fn random_subset(rng: &mut impl RngCore, set: &[Self], amount: usize) -> BTreeSet<Self> {
        set.choose_multiple(rng, amount).cloned().collect()
    }

    pub fn primary_color(&self) -> Color {
        match self {
            Self::Rock => Color::hsl(25.0, 0.75, 0.33),
            Self::Water => Color::hsl(218.0, 0.78, 0.25),
            Self::Air => Color::hsl(230.0, 0.03, 0.44),
            Self::Paper => Color::hsl(312.0, 0.45, 0.45),
            Self::Earth => Color::hsl(149.0, 0.84, 0.39),
            Self::Scissors => Color::hsl(198.0, 0.44, 0.58),
            Self::Fire => Color::hsl(2.0, 0.76, 0.41),
        }
    }

    pub fn key(&self) -> String {
        format!("{self}").to_lowercase()
    }
}

#[derive(Clone, Debug)]
#[derive(Deref, DerefMut)]
#[derive(Component, Reflect)]
pub struct Guess(Element);

impl Guess {
    pub fn new(element: Element) -> Self {
        Guess(element)
    }
}

#[derive(Clone, Debug, Default)]
#[derive(Add, AddAssign, Mul, MulAssign, Sum)]
pub struct Payout {
    pub damage_to_me: i32,
    pub damage_to_enemy: i32,
}

impl Payout {
    pub fn invert(self) -> Self {
        Self {
            damage_to_enemy: self.damage_to_me,
            damage_to_me: self.damage_to_enemy,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Action {
    pub guess: Element,
    pub enchantment: Option<Aspect>,
    pub augmentation: Option<Augmentation>,
}

impl Action {
    pub fn new(
        guess: Element,
        enchantment: Option<Aspect>,
        augmentation: Option<Augmentation>,
    ) -> Self {
        Self {
            guess,
            augmentation,
            enchantment,
        }
    }

    pub fn with_augmentation(mut self, augmentation: Augmentation) -> Self {
        self.augmentation = Some(augmentation);
        self
    }

    pub fn with_enchantment(mut self, aspect: Aspect) -> Self {
        self.enchantment = Some(aspect);
        self
    }
}

impl From<Element> for Action {
    fn from(element: Element) -> Self {
        Self {
            guess: element,
            augmentation: None,
            enchantment: None,
        }
    }
}

impl From<Guess> for Action {
    fn from(guess: Guess) -> Self {
        Self {
            guess: guess.0,
            augmentation: None,
            enchantment: None,
        }
    }
}

pub struct PlayerOutcome {
    pub damage: i32,
    pub next_combo: Option<Combo>,
    pub next_stagger: Option<Stagger>,
}

#[derive(Component)]
pub struct Outcome {
    pub p1_action: Action,
    pub p1_outcome: PlayerOutcome,
    pub p2_action: Action,
    pub p2_outcome: PlayerOutcome,
}

impl Outcome {
    pub fn new(
        table: &ElementTable,
        p1_action: Action,
        p1_combo: Option<&Combo>,
        p1_stagger: Option<&Stagger>,
        p2_action: Action,
        p2_combo: Option<&Combo>,
        p2_stagger: Option<&Stagger>,
    ) -> Self {
        let p1_aspects = if let Some(enchantment) = p1_action.enchantment {
            vec![p1_action.guess, *enchantment]
        } else {
            vec![p1_action.guess]
        };
        let p2_aspects = if let Some(enchantment) = p2_action.enchantment {
            vec![p2_action.guess, *enchantment]
        } else {
            vec![p2_action.guess]
        };

        let payout: Payout = p1_aspects
            .into_iter()
            .cartesian_product(p2_aspects)
            .map(|(element1, element2)| table.evaluate(element1, element2))
            .cloned()
            .sum();

        let armored_payout = Payout {
            damage_to_me: if let Some(Augmentation::Armored) = p1_action.augmentation {
                if payout.damage_to_me.is_positive() {
                    1
                } else {
                    payout.damage_to_me
                }
            } else {
                payout.damage_to_me
            },
            damage_to_enemy: if let Some(Augmentation::Armored) = p2_action.augmentation {
                if payout.damage_to_enemy.is_positive() {
                    1
                } else {
                    payout.damage_to_enemy
                }
            } else {
                payout.damage_to_enemy
            },
        };
        let is_parry_throw = p1_action.guess == p2_action.guess;
        let p1_element_has_parry = matches!(p1_action.augmentation, Some(Augmentation::Parry));
        let p2_element_has_parry = matches!(p2_action.augmentation, Some(Augmentation::Parry));
        let parry_payout = if is_parry_throw {
            Payout {
                damage_to_me: if p1_element_has_parry {
                    armored_payout.damage_to_me - 1
                } else {
                    armored_payout.damage_to_me
                },
                damage_to_enemy: if p2_element_has_parry {
                    armored_payout.damage_to_enemy - 1
                } else {
                    armored_payout.damage_to_enemy
                },
            }
        } else {
            armored_payout
        };
        let p1_stagger = if is_parry_throw && p2_element_has_parry && p1_stagger.is_none() {
            Some(Stagger)
        } else {
            None
        };
        let p2_stagger = if is_parry_throw && p1_element_has_parry && p2_stagger.is_none() {
            Some(Stagger)
        } else {
            None
        };

        let p1_element_has_combo = matches!(p1_action.augmentation, Some(Augmentation::Combo));
        let p1_has_combo = p1_combo.is_some();
        let does_p1_continue_combo = if parry_payout.damage_to_enemy >= parry_payout.damage_to_me {
            Some(Combo)
        } else {
            None
        };
        let does_p2_continue_combo = if parry_payout.damage_to_me >= parry_payout.damage_to_enemy {
            Some(Combo)
        } else {
            None
        };
        let p2_element_has_combo = matches!(p2_action.augmentation, Some(Augmentation::Combo));
        let p2_has_combo = p2_combo.is_some();

        // TODO: first cast with combo seems to double
        let combo_payout = Payout {
            damage_to_me: if p2_has_combo && p2_element_has_combo {
                parry_payout.damage_to_me * 2
            } else {
                parry_payout.damage_to_me
            },
            damage_to_enemy: if p1_has_combo && p1_element_has_combo {
                parry_payout.damage_to_enemy * 2
            } else {
                parry_payout.damage_to_enemy
            },
        };

        Outcome {
            p1_action,
            p1_outcome: PlayerOutcome {
                damage: combo_payout.damage_to_me,
                next_combo: does_p1_continue_combo,
                next_stagger: p1_stagger,
            },
            p2_action,
            p2_outcome: PlayerOutcome {
                damage: combo_payout.damage_to_enemy,
                next_combo: does_p2_continue_combo,
                next_stagger: p2_stagger,
            },
        }
    }
}

/// Augmenting an element gives it an extra conditional effect when used.
/// Each element may only have one augmentation.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
#[derive(Display)]
#[derive(Reflect)]
pub enum Augmentation {
    /// The player takes 1 less HP damage when losing with this move.
    Armored,
    /// When this element competes against itself, heal 1 and the opponent must repeat its action.
    Parry,
    /// Combo provides advantages to chained successful Combo moves.
    /// Whenever a Combo move wins a round, the player gain Combo.
    /// The next time the player uses a Combo move, if it wins, the damage is doubled.
    Combo,
}

impl Augmentation {
    pub const ALL: [Self; 3] = [Self::Armored, Self::Parry, Self::Combo];

    pub fn get_description(&self, element: Element) -> String {
        match self {
            Augmentation::Armored => format!("When {element} takes damage, it takes only 1 damage."),
            Augmentation::Parry => format!("When {element} meets {element}, heal 1. The opponent must repeat {element} next turn."),
            Augmentation::Combo => format!("{element} gains Combo. Combo provides advantages to chained successful Combo moves. Whenever a Combo move wins a round, the player gains Combo. The next time the player uses a Combo move, if it wins, the damage is doubled. If it loses, the player loses Combo."),
        }
    }

    pub fn primary_color(&self) -> Color {
        use bevy::color::palettes::css;
        match self {
            Self::Armored => css::LIGHT_STEEL_BLUE.with_alpha(0.4),
            Self::Parry => css::CHARTREUSE.with_alpha(0.4),
            Self::Combo => css::RED.with_alpha(0.4),
        }
        .into()
    }
}

/// An Aspect is an additional composite element which influences how the element interacts with enemy elements.
/// Whenever an element with an Aspect is used, both the original and Aspect elements are compared against the enemy element,
/// and the sum of the two results is returned.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
#[derive(Constructor, Deref, DerefMut)]
#[derive(Reflect)]
pub struct Aspect(Element);

impl Aspect {
    pub fn get_description(&self, element: Element) -> String {
        let aspect = self.0;
        format!("{element} gains the Aspect of {aspect}. All results are computed as the sum of the results of those two elements.")
    }

    pub fn primary_color(&self) -> Color {
        self.0.primary_color().with_alpha(0.5)
    }
}

#[derive(Clone, Copy)]
#[derive(Component)]
pub struct Combo;

#[derive(Clone, Copy)]
#[derive(Component)]
pub struct Stagger;

#[derive(Clone, Debug)]
#[derive(Component)]
pub struct ElementTable(BTreeMap<(Element, Element), Payout>);

impl Default for ElementTable {
    fn default() -> Self {
        let win = Payout {
            damage_to_enemy: 1,
            damage_to_me: 0,
        };
        let loss = Payout {
            damage_to_enemy: 0,
            damage_to_me: 1,
        };
        let draw = Payout {
            damage_to_enemy: 0,
            damage_to_me: 0,
        };

        ElementTable(
            Element::ALL
                .iter()
                .cloned()
                .cartesian_product(Element::ALL.iter().cloned())
                .map(|(element1, element2)| {
                    (
                        (element1, element2),
                        match element1.naive_compare(element2) {
                            Ordering::Less => loss.clone(),
                            Ordering::Equal => draw.clone(),
                            Ordering::Greater => win.clone(),
                        },
                    )
                })
                .collect(),
        )
    }
}

impl ElementTable {
    pub fn update(&mut self, my_element: Element, enemy_element: Element, delta_payout: Payout) {
        if let Some(payout) = self.0.get_mut(&(my_element, enemy_element)) {
            *payout += delta_payout;
        }
    }

    pub fn double(&mut self, my_element: Element, enemy_element: Element) {
        if let Some(payout) = self.0.get_mut(&(my_element, enemy_element)) {
            *payout *= 2;
        }
    }

    pub fn evaluate(&self, me: Element, enemy: Element) -> &Payout {
        self.0.get(&(me, enemy)).unwrap()
    }
}
