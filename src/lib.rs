#![doc = include_str!("../README.md")]

pub mod error;
mod component;
mod parser;
mod property;
mod selector;
mod stylesheet;
pub mod system;

use bevy::{
    asset::AssetSet,
    ecs::system::SystemState,
    prelude::{
        AddAsset, Button, Component, CoreSet, Entity, IntoSystemConfig, IntoSystemSetConfig,
        Plugin, Query, With,
    },
    text::Text,
    ui::{BackgroundColor, Interaction, Node, Style, UiImage},
};

use property::StyleSheetState;
use stylesheet::StyleSheetLoader;

use system::{ComponentFilterRegistry, PrepareParams};

pub use component::{Class, StyleSheet};
pub use property::{Property, PropertyToken, PropertyValues};
pub use selector::{Selector, SelectorElement};
pub use stylesheet::{StyleRule, StyleSheetAsset};

/// use `tomt_bevycss::prelude::*;` to import common components, and plugins and utility functions.
pub mod prelude {
    pub use super::{
        BevyCssPlugin,
        RegisterComponentSelector,
        RegisterProperty,
        error::BevyCssError,
        component::{Class, PseudoClass, StyleSheet},
        stylesheet::StyleSheetAsset,
    };
}

/// Plugin which add all types, assets, systems and internal resources needed by `tomt_bevycss`.
/// You must add this plugin in order to use `tomt_bevycss`.
#[derive(Default)]
pub struct BevyCssPlugin {
    hot_reload: bool,
}

impl BevyCssPlugin {
    pub fn with_hot_reload() -> BevyCssPlugin {
        BevyCssPlugin { hot_reload: true }
    }
}

impl Plugin for BevyCssPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        use system::sets::*;

        app.register_type::<Class>()
            .register_type::<StyleSheet>()
            .add_asset::<StyleSheetAsset>()
            .configure_set(
                BevyCssSet::Prepare
                    .in_base_set(CoreSet::PreUpdate)
            )
            .configure_set(
                BevyCssSet::Apply
                    .in_base_set(CoreSet::PreUpdate)
                    .after(BevyCssSet::Prepare),
            )
            .configure_set(
                BevyCssSet::Cleanup
                    .in_base_set(CoreSet::PostUpdate)
            )
            .init_resource::<StyleSheetState>()
            .init_resource::<ComponentFilterRegistry>()
            .init_asset_loader::<StyleSheetLoader>()
            .add_system(
                system::prepare
                    .in_set(BevyCssSet::Prepare)
            )
            .add_system(
                system::clear_state
                    .in_set(BevyCssSet::Cleanup)
            );

        let prepared_state = PrepareParams::new(&mut app.world);
        app.insert_resource(prepared_state);

        register_component_selector(app);
        register_properties(app);

        if self.hot_reload {
            app.configure_set(
                BevyCssHotReload
                    .after(AssetSet::AssetEvents)
                    .before(CoreSet::Last),
            )
            .add_system(
                system::hot_reload_style_sheets
                    .in_base_set(BevyCssHotReload)
            );
        }
    }
}

fn register_component_selector(app: &mut bevy::prelude::App) {
    app.register_component_selector::<BackgroundColor>("background-color");
    app.register_component_selector::<Text>("text");
    app.register_component_selector::<Button>("button");
    app.register_component_selector::<Node>("node");
    app.register_component_selector::<Style>("style");
    app.register_component_selector::<UiImage>("ui-image");
    app.register_component_selector::<Interaction>("interaction");
}

fn register_properties(app: &mut bevy::prelude::App) {
    use property::impls::*;

    app.register_property::<DisplayProperty>();
    app.register_property::<PositionTypeProperty>();
    app.register_property::<DirectionProperty>();
    app.register_property::<FlexDirectionProperty>();
    app.register_property::<FlexWrapProperty>();
    app.register_property::<AlignItemsProperty>();
    app.register_property::<AlignSelfProperty>();
    app.register_property::<AlignContentProperty>();
    app.register_property::<JustifyContentProperty>();
    app.register_property::<OverflowProperty>();

    app.register_property::<LeftProperty>();
    app.register_property::<RightProperty>();
    app.register_property::<TopProperty>();
    app.register_property::<BottomProperty>();
    app.register_property::<WidthProperty>();
    app.register_property::<HeightProperty>();
    app.register_property::<MinWidthProperty>();
    app.register_property::<MinHeightProperty>();
    app.register_property::<MaxWidthProperty>();
    app.register_property::<MaxHeightProperty>();
    app.register_property::<FlexBasisProperty>();
    app.register_property::<FlexGrowProperty>();
    app.register_property::<FlexShrinkProperty>();
    app.register_property::<AspectRatioProperty>();

    app.register_property::<MarginProperty>();
    app.register_property::<PaddingProperty>();
    app.register_property::<BorderProperty>();

    app.register_property::<FontColorProperty>();
    app.register_property::<FontProperty>();
    app.register_property::<FontSizeProperty>();
    app.register_property::<TextAlignProperty>();
    app.register_property::<TextContentProperty>();

    app.register_property::<BackgroundColorProperty>();
}

/// Utility trait which adds the [`register_component_selector`](RegisterComponentSelector::register_component_selector)
/// function on [`App`](bevy::prelude::App) to add a new component selector.
///
/// You can register any component you want and name it as you like.
/// It's advised to use `lower-case` and `kebab-case` to match CSS coding style.
///
/// # Examples
///
/// ```
/// # use bevy::prelude::*;
/// # use tomt_bevycss::prelude::*;
/// #
/// # #[derive(Component)]
/// # struct MyFancyComponentSelector;
/// #
/// # fn some_main() {
/// #    let mut app = App::new();
/// #    app.add_plugins(DefaultPlugins).add_plugin(BevyCssPlugin::default());
/// // You may use it as selector now, like
/// // fancy-pants {
/// //      background-color: pink;
/// // }
/// app.register_component_selector::<MyFancyComponentSelector>("fancy-pants");
/// # }
/// ```

pub trait RegisterComponentSelector {
    fn register_component_selector<T>(&mut self, name: &'static str) -> &mut Self
    where
        T: Component;
}

impl RegisterComponentSelector for bevy::prelude::App {
    fn register_component_selector<T>(&mut self, name: &'static str) -> &mut Self
    where
        T: Component,
    {
        let system_state = SystemState::<Query<Entity, With<T>>>::new(&mut self.world);
        let boxed_state = Box::new(system_state);

        self.world
            .get_resource_or_insert_with::<ComponentFilterRegistry>(|| {
                ComponentFilterRegistry(Default::default())
            })
            .0
            .insert(name, boxed_state);

        self
    }
}

/// Utility trait which adds the [`register_property`](RegisterProperty::register_property) function
/// on [`App`](bevy::prelude::App) to add a [`Property`] parser.
///
/// You need to register only custom properties which implements [`Property`] trait.
pub trait RegisterProperty {
    fn register_property<T>(&mut self) -> &mut Self
    where
        T: Property + 'static;
}

impl RegisterProperty for bevy::prelude::App {
    fn register_property<T>(&mut self) -> &mut Self
    where
        T: Property + 'static,
    {
        self.add_system(
            T::apply_system
                .in_set(system::sets::BevyCssSet::Apply)
        );
        self
    }
}
