use std::any::TypeId;

use bevy::{
    app::{App, Startup},
    core::{DebugName, Name},
    core_pipeline::core_2d::Camera2dBundle,
    ecs::{
        entity::Entity, query::Without, reflect::AppTypeRegistry, system::Commands, world::World,
    },
    prelude::ReflectComponent,
    reflect::{NamedField, Reflect, ReflectRef, TypeInfo, TypeRegistry, VariantInfo},
    ui::{node_bundles::NodeBundle, Node},
    DefaultPlugins,
};
use bevy_dioxus::{colors::*, prelude::*};
use bevy_mod_picking::DefaultPickingPlugins;

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, DioxusUiPlugin, DefaultPickingPlugins))
        .add_systems(Startup, |mut commands: Commands| {
            commands.spawn(DioxusUiBundle {
                dioxus_ui_root: DioxusUiRoot(Editor),
                node_bundle: NodeBundle::default(),
            });
            commands.spawn((Camera2dBundle::default(), Name::new("Camera")));
        })
        .run();
}

#[component]
fn Editor(cx: Scope) -> Element {
    // TODO: When selected entity is despawned, need to reset this to None
    let selected_entity = use_state_sendable(cx, || Option::<Entity>::None);

    render! {
        node {
            width: "100vw",
            height: "100vh",
            justify_content: "space_between",
            SceneTree { selected_entity: selected_entity }
            EntityInspector { selected_entity: selected_entity }
        }
    }
}

#[component]
fn SceneTree<'a>(cx: Scope, selected_entity: &'a UseStateSendable<Option<Entity>>) -> Element {
    let entities = use_query_filtered::<(Entity, DebugName), Without<Node>>(cx);
    let entities = entities.query();
    let mut entities = entities.into_iter().collect::<Vec<_>>();
    entities.sort_by_key(|(entity, _)| *entity);

    let system_scheduler = use_system_scheduler(cx);

    render! {
        node {
            onclick: move |_| selected_entity.write(None),
            flex_direction: "column",
            if entities.is_empty() {
                rsx! { "No entities exist" }
            } else {
                rsx! {
                    for (entity, name) in entities {
                        Button {
                            onclick: move |event: Event<PointerButton>| if *event.data == PointerButton::Primary {
                                if Some(entity) == *selected_entity.read() {
                                    selected_entity.write(None);
                                } else {
                                    selected_entity.write(Some(entity));
                                }
                                event.stop_propagation();
                            },
                            base_color: if Some(entity) == *selected_entity.read() { Some(VIOLET_700) } else { None },
                            click_color: if Some(entity) == *selected_entity.read() { Some(VIOLET_400) } else { None },
                            hover_color: if Some(entity) == *selected_entity.read() { Some(VIOLET_500) } else { None },
                            match name.name {
                                Some(name) => format!("{name}"),
                                _ => format!("Entity ({:?})", name.entity)
                            }
                        }
                    }
                }
            }
            Button {
                onclick: move |event: Event<PointerButton>| if *event.data == PointerButton::Primary {
                    system_scheduler.schedule({
                        let selected_entity = (*selected_entity).clone();
                        move |world: &mut World| {
                            let new_entity = world.spawn_empty();
                            selected_entity.write(Some(new_entity.id()));
                        }
                    });
                    event.stop_propagation();
                },
                text { text: "Spawn Entity", text_size: "18" }
            }
        }
    }
}

#[component]
fn EntityInspector<'a>(
    cx: Scope,
    selected_entity: &'a UseStateSendable<Option<Entity>>,
) -> Element {
    let world = use_world(cx);
    let type_registry = use_resource::<AppTypeRegistry>(cx).read();
    let components = selected_entity
        .read()
        .map(|selected_entity| {
            let entity_ref = world.get_entity(selected_entity).unwrap();
            let mut components = entity_ref
                .archetype()
                .components()
                .map(|component_id| {
                    let component_info = world.components().get_info(component_id).unwrap();
                    let type_info = component_info
                        .type_id()
                        .and_then(|type_id| type_registry.get_type_info(type_id));
                    let (_, name) = component_info.name().rsplit_once("::").unwrap();
                    let (crate_name, _) = component_info.name().split_once("::").unwrap();
                    (name, crate_name, type_info)
                })
                .collect::<Vec<_>>();
            components.sort_by_key(|(name, _, _)| *name);
            components
        })
        .unwrap_or_default();

    render! {
        if selected_entity.read().is_none() {
            rsx! {
                node {
                    margin: "8",
                    "Select an entity to view its components"
                }
            }
        } else {
            rsx! {
                node {
                    flex_direction: "column",
                    margin: "8",
                    text { text: "Entity Inspector", text_size: "24" }
                    for (name, crate_name, type_info) in components {
                        node {
                            flex_direction: "column",
                            margin_bottom: "6",
                            node {
                                column_gap: "6",
                                align_items: "baseline",
                                text { text: name, text_size: "18" }
                                text { text: crate_name, text_size: "14", text_color: NEUTRAL_400 }
                            }
                            if let Some(type_info) = type_info {
                                rsx! { ComponentInspector { entity: selected_entity.read().unwrap(), type_info: type_info } }
                            }
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn ComponentInspector<'a>(cx: Scope, entity: Entity, type_info: &'a TypeInfo) -> Element {
    render! {
        match type_info {
            TypeInfo::Struct(info) => rsx! {
                for field in info.iter() {
                    InspectorFieldValue {
                        entity: *entity,
                        field: field,
                        type_info: type_info,
                    }
                }
            },
            TypeInfo::TupleStruct(info) => rsx! {
                info.iter().map(|t| format!("{}: {}", t.index(), t.type_path())).collect::<Vec<String>>().join(", ")
            },
            TypeInfo::Tuple(info) => rsx! { info.type_path().to_string() },
            TypeInfo::List(info) => rsx! { info.type_path().to_string() },
            TypeInfo::Array(info) => rsx! { info.type_path().to_string() },
            TypeInfo::Map(info) => rsx! { info.type_path().to_string() },
            TypeInfo::Enum(info) => {
                rsx! {
                    for variant in info.iter() {
                        match variant {
                            VariantInfo::Struct(variant_info) => format!(
                                "{} {{\n{}\n}}",
                                variant.name(),
                                variant_info.iter().map(|t| format!("  {}: {},", t.name(), t.type_path())).collect::<Vec<String>>().join("\n")
                            ),
                            VariantInfo::Tuple(variant_info) => format!(
                                "{}({})",
                                variant.name(),
                                variant_info.iter().map(|t| t.type_path().to_string()).collect::<Vec<String>>().join(", ")
                            ),
                            VariantInfo::Unit(_) => variant.name().to_string(),
                        }
                    }
                }
            },
            TypeInfo::Value(info) => rsx! { info.type_path().to_string() },
        }
    }
}

fn get_reflect_value<'a, T: Reflect + Copy>(
    world: &'a World,
    type_registry: &'a TypeRegistry,
    entity: Entity,
    type_info: &'a TypeInfo,
    field_name: &'a str,
) -> &'a T {
    type_registry
        .get(type_info.type_id())
        .and_then(|registration| registration.data::<ReflectComponent>())
        .and_then(|reflect_component| reflect_component.reflect(world.entity(entity)))
        .and_then(|data| {
            if let ReflectRef::Struct(data) = data.reflect_ref() {
                data.field(field_name)
                    .and_then(|field| field.downcast_ref::<T>())
            } else {
                None
            }
        })
        .unwrap()
}

#[component]
fn InspectorFieldBool<'a>(
    cx: Scope,
    entity: Entity,
    field_name: &'a str,
    type_info: &'a TypeInfo,
) -> Element {
    let world = use_world(cx);
    let type_registry = use_resource::<AppTypeRegistry>(cx).read();

    let value = get_reflect_value::<bool>(world, &type_registry, *entity, type_info, field_name);

    render! {
        text { text: "{field_name}: {value} (bool)", text_color: AMBER_100 }
    }
}

#[component]
fn InspectorFieldF32<'a>(
    cx: Scope,
    entity: Entity,
    field_name: &'a str,
    type_info: &'a TypeInfo,
) -> Element {
    let world = use_world(cx);
    let type_registry = use_resource::<AppTypeRegistry>(cx).read();

    let value = get_reflect_value::<f32>(world, &type_registry, *entity, type_info, field_name);

    render! {
        text { text: "{field_name}: {value} (f32)", text_color: AMBER_100 }
    }
}

#[component]
fn InspectorFieldValue<'a>(
    cx: Scope,
    entity: Entity,
    field: &'a NamedField,
    type_info: &'a TypeInfo,
) -> Element {
    render! {
        if field.type_id() == TypeId::of::<bool>() {
            rsx! {
                InspectorFieldBool {
                    entity: *entity,
                    field_name: field.name(),
                    type_info: type_info,
                }
            }
        } else if field.type_id() == TypeId::of::<f32>() {
            rsx! {
                InspectorFieldF32 {
                    entity: *entity,
                    field_name: field.name(),
                    type_info: type_info,
                }
            }
        } else {
            rsx! {
                "{field.name()}: NOT SUPPORTED ({field.type_path()})"
            }
        }
    }
}

#[allow(non_snake_case)]
fn Button<'a>(cx: Scope<'a, ButtonProps<'a>>) -> Element<'a> {
    let clicked = use_state(cx, || false);
    let hovered = use_state(cx, || false);
    let background_color = if **clicked {
        cx.props.click_color.unwrap_or(NEUTRAL_500)
    } else if **hovered {
        cx.props.hover_color.unwrap_or(NEUTRAL_600)
    } else {
        cx.props.base_color.unwrap_or(NEUTRAL_800)
    };

    render! {
        node {
            onclick: move |event| cx.props.onclick.call(event),
            onclick_down: |event| if *event.data == PointerButton::Primary { clicked.set(true) },
            onclick_up: |event| if *event.data == PointerButton::Primary { clicked.set(false) },
            onmouse_enter: |_| hovered.set(true),
            onmouse_exit: |_| { hovered.set(false); clicked.set(false) },
            padding: "8",
            background_color: background_color,
            &cx.props.children
        }
    }
}

#[derive(Props)]
struct ButtonProps<'a> {
    onclick: EventHandler<'a, Event<PointerButton>>,
    base_color: Option<&'a str>,
    click_color: Option<&'a str>,
    hover_color: Option<&'a str>,
    children: Element<'a>,
}
