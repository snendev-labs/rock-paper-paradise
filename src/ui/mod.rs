use std::{
    collections::{BTreeMap, BTreeSet},
    marker::PhantomData,
};

use bevy::{
    asset::AssetServer,
    ecs::system::{StaticSystemParam, SystemParam},
    prelude::{
        Added, App, BuildChildren, Camera2d, Changed, ChildBuilder, Commands, Component,
        DespawnRecursiveExt, Entity, FromWorld, Handle, Image, ImageNode, IntoSystemConfigs,
        IntoSystemSetConfigs, Node, Plugin, PositionType, Query, RemovedComponents, Resource,
        Single, Startup, SystemSet, Text, TextFont, Update, Val, World,
    },
};

mod bonus;
pub use bonus::*;

mod game;
pub use game::*;

mod game_over;
pub use game_over::*;

mod home;
pub use home::*;

use crate::{Action, Aspect, Augmentation, Element};

pub struct UIPlugin;

impl Plugin for UIPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<RpsGlyphs>();
        app.configure_sets(
            Update,
            (UISystems::Watch, UISystems::Unmount, UISystems::Mount).chain(),
        );
        app.add_systems(Startup, Self::spawn_ui);
        app.add_systems(Update, Self::watch_screen_changes.in_set(UISystems::Watch));
        app.add_plugins((
            GameUIPlugin,
            UIComponentPlugin::<HomeMenuUIComponent>::default(),
            UIComponentPlugin::<GameUIComponent>::default(),
            UIComponentPlugin::<BonusUIComponent>::default(),
            UIComponentPlugin::<GameOverUIComponent>::default(),
        ));
    }
}

impl UIPlugin {
    fn spawn_ui(mut commands: Commands) {
        commands.spawn(Camera2d);
        commands.spawn(AppScreen::default());
    }

    fn watch_screen_changes(
        mut commands: Commands,
        app_screen: Option<Single<(Entity, &AppScreen), Changed<AppScreen>>>,
    ) {
        let Some(app_screen) = app_screen else {
            return;
        };
        let (root, app_screen) = *app_screen;
        commands
            .entity(root)
            .remove::<HomeMenuUIComponent>()
            .remove::<GameUIComponent>()
            .remove::<BonusUIComponent>()
            .remove::<GameOverUIComponent>();
        app_screen.build_ui(&mut commands, root);
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[derive(SystemSet)]
pub enum UISystems {
    Watch,
    Mount,
    Unmount,
}

pub fn text_bundle(text: impl Into<String>, font: TextFont, node: Node) -> (Text, TextFont, Node) {
    (Text::new(text), font, node)
}

pub fn image_bundle(image: Handle<Image>, node: Node) -> (ImageNode, Node) {
    (ImageNode::new(image), node)
}

#[derive(Component)]
pub struct LocalPlayer;

pub trait UIComponent: Component + Clone {
    type Params: SystemParam;

    fn build_ui(self, builder: &mut ChildBuilder<'_>, params: &StaticSystemParam<Self::Params>);
}

struct UIComponentPlugin<C> {
    _marker: PhantomData<C>,
}

impl<C> Plugin for UIComponentPlugin<C>
where
    C: UIComponent + Send + Sync + 'static,
{
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                Self::unmount_components.in_set(UISystems::Unmount),
                Self::mount_components.in_set(UISystems::Mount),
            ),
        );
    }
}

impl<C> UIComponentPlugin<C> {
    fn mount_components(
        mut commands: Commands,
        added_components: Query<(Entity, &C), Added<C>>,
        params: StaticSystemParam<C::Params>,
    ) where
        C: UIComponent + Clone,
    {
        for (root, component) in &added_components {
            commands.entity(root).with_children(|builder| {
                component.clone().build_ui(builder, &params);
            });
        }
    }

    fn unmount_components(
        mut commands: Commands,
        mut removed_components: RemovedComponents<C>,
        entities: Query<Entity>,
    ) where
        C: UIComponent,
    {
        for root in removed_components.read() {
            if entities.contains(root) {
                commands.entity(root).despawn_descendants();
            }
        }
    }
}

impl<C> Default for UIComponentPlugin<C> {
    fn default() -> Self {
        Self {
            _marker: PhantomData,
        }
    }
}

#[derive(Debug)]
#[derive(Component)]
#[require(Node(Self::node))]
pub enum AppScreen {
    HomeMenu,
    InGame(GameUIComponent),
    ProvidingBonus(BonusUIComponent),
    GameOver(GameOverUIComponent),
}

impl AppScreen {
    fn node() -> Node {
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(0.),
            bottom: Val::Px(0.),
            left: Val::Px(0.),
            right: Val::Px(0.),
            ..Default::default()
        }
    }

    fn build_ui(&self, commands: &mut Commands, root: Entity) {
        match self {
            AppScreen::HomeMenu => {
                commands.entity(root).insert(HomeMenuUIComponent);
            }
            AppScreen::InGame(ui_child) => {
                commands.entity(root).insert(ui_child.clone());
            }
            AppScreen::ProvidingBonus(ui_child) => {
                commands.entity(root).insert(ui_child.clone());
            }
            AppScreen::GameOver(ui_child) => {
                commands.entity(root).insert(ui_child.clone());
            }
        }
    }
}

impl Default for AppScreen {
    fn default() -> Self {
        Self::HomeMenu
    }
}

#[derive(Debug)]
#[derive(Resource)]
pub struct RpsGlyphs(BTreeMap<Action, Handle<Image>>);

impl RpsGlyphs {
    pub fn get_image(&self, action: &Action) -> Option<&Handle<Image>> {
        Some(self.0.get(action).unwrap_or_else(|| {
            let element = action.guess;
            let aspect = action.enchantment;
            let augmentation = action.augmentation;
            panic!("{element}-{aspect:?}-{augmentation:?}.png should be in RpsGlyphs");
        }))
    }
}

impl FromWorld for RpsGlyphs {
    fn from_world(world: &mut World) -> Self {
        let asset_server = world.resource_mut::<AssetServer>();
        let mut glyph_map = BTreeMap::default();
        for element in Element::ALL {
            glyph_map.insert(
                Action::new(element, None, None),
                asset_server.load(format!("{}.png", element.key())),
            );
            for augmentation in Augmentation::ALL {
                glyph_map.insert(
                    Action::new(element, None, Some(augmentation)),
                    asset_server.load(format!(
                        "{}-{}.png",
                        element.key(),
                        augmentation.to_string().to_lowercase(),
                    )),
                );
            }
            let aspects = {
                let mut aspects = BTreeSet::from(Element::ALL);
                aspects.remove(&element);
                aspects
            };
            for aspect in aspects {
                glyph_map.insert(
                    Action::new(element, Some(Aspect::new(aspect)), None),
                    asset_server.load(format!("{}-{}.png", element.key(), aspect.key())),
                );
                for augmentation in Augmentation::ALL {
                    glyph_map.insert(
                        Action::new(element, Some(Aspect::new(aspect)), Some(augmentation)),
                        asset_server.load(format!(
                            "{}-{}-{}.png",
                            element.key(),
                            aspect.key(),
                            augmentation.to_string().to_lowercase(),
                        )),
                    );
                }
            }
        }
        Self(glyph_map)
    }
}
