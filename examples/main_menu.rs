use bevy::{app::AppExit, prelude::*};
use bevy_dioxus::{
    colors::*,
    prelude::{Event as DioxusEvent, *},
};
use bevy_mod_picking::DefaultPickingPlugins;

fn main() {
    App::new()
        .add_state::<MenuState>()
        .add_plugins((DefaultPlugins, DioxusUiPlugin, DefaultPickingPlugins))
        .add_systems(Startup, |mut commands: Commands| {
            commands.spawn(DioxusUiBundle {
                dioxus_ui_root: DioxusUiRoot(AppRoot),
                node_bundle: NodeBundle::default(),
            });
            commands.spawn((Camera2dBundle::default(), Name::new("Camera")));
        })
        .add_systems(OnEnter(MenuState::Game), on_enter_game)
        .add_systems(OnExit(MenuState::Game), on_exit_game)
        .add_systems(Update, tick_game_timer.run_if(in_state(MenuState::Game)))
        .run();
}

/// Timer added when in game, returns to the main menu after 3 seconds
#[derive(Resource)]
struct GameTimer(Timer);
impl GameTimer {
    pub fn new() -> Self {
        Self(Timer::from_seconds(3., TimerMode::Once))
    }
}

/// Marker component for despawning everything from the game when returning to the menu
#[derive(Component)]
struct OnGame;

/// System run when entering game state. Shows a message using native bevy_ui.
fn on_enter_game(mut commands: Commands) {
    commands.insert_resource(GameTimer::new());
    commands
        .spawn((
            OnGame,
            NodeBundle {
                style: Style {
                    width: Val::Percent(100.),
                    height: Val::Percent(100.),
                    position_type: PositionType::Absolute,
                    top: Val::Px(0.),
                    left: Val::Px(0.),
                    bottom: Val::Px(0.),
                    right: Val::Px(0.),
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    ..default()
                },
                ..default()
            },
        ))
        .with_children(|p| {
            p.spawn(TextBundle::from_section(
                "In Game. bevy_dioxus ui is hidden. Will return in 3 seconds.",
                TextStyle::default(),
            ));
        });
}

/// System run when exiting game state. Removes the game timer resource and despawns all game entities.
fn on_exit_game(mut commands: Commands, query: Query<Entity, With<OnGame>>) {
    commands.remove_resource::<GameTimer>();
    for entity in query.iter() {
        commands.entity(entity).despawn_recursive();
    }
}

/// System which runs in game state. It ticks the `GameTimer` resource, and returns to the main menu when complete.
fn tick_game_timer(
    time: Res<Time>,
    mut game_timer: ResMut<GameTimer>,
    mut next_state: ResMut<NextState<MenuState>>,
) {
    game_timer.0.tick(time.delta());
    if game_timer.0.finished() {
        next_state.set(MenuState::Main);
    }
}

/// State for the main menu/game.
#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States)]
pub enum MenuState {
    #[default]
    Main,
    Game,
    Settings,
    Credits,
}

/// Actions which can be performed by menu buttons.
#[derive(Clone, Copy)]
enum MenuButtonAction {
    ChangeState(MenuState),
    Exit,
}

const PANEL_PADDING: &str = "20";
const PANEL_MIN_WIDTH: &str = "200";
const MAIN_MENU_BUTTON_SPACER: &str = "10";

/// The root dioxus component
#[component]
fn AppRoot(cx: Scope) -> Element {
    let menu_state = use_resource::<State<MenuState>>(cx).get();

    render! {
        match menu_state {
            MenuState::Main => rsx! {
                MainMenu {}
            },
            MenuState::Settings => rsx! {
                SettingsMenu {}
            },
            MenuState::Credits => rsx! {
                CreditsMenu {}
            },
            // Do not render anything when in the game. In a real game, you might add a score card, FPS or any in-game
            // menus here.
            MenuState::Game => rsx! { "" },
        }
    }
}

/// Generic props for components that only accept a children prop
#[derive(Props)]
struct ChildrenProps<'a> {
    children: Element<'a>,
}

/// Main menu dioxus component
#[component]
fn MainMenu(cx: Scope) -> Element {
    render! {
        MenuPanel {
            title: "Main Menu".to_string(),
            MenuPanelBody {
                MenuButton {
                    margin_bottom: MAIN_MENU_BUTTON_SPACER,
                    action: MenuButtonAction::ChangeState(MenuState::Game),
                    "Play"
                }
                MenuButton {
                    margin_bottom: MAIN_MENU_BUTTON_SPACER,
                    action: MenuButtonAction::ChangeState(MenuState::Settings),
                    "Settings"
                }
                MenuButton {
                    margin_bottom: MAIN_MENU_BUTTON_SPACER,
                    action: MenuButtonAction::ChangeState(MenuState::Credits),
                    "Credits"
                }
                MenuButton {
                    action: MenuButtonAction::Exit,
                    "Quit"
                }
            }
        }
    }
}

/// Settings menu dioxus component
#[component]
fn SettingsMenu(cx: Scope) -> Element {
    render! {
        MenuPanel {
            title: "Settings".to_string(),
            MenuPanelBody {
                "TODO: Settings menu"
            },
            MenuPanelFooter {
                MenuButton {
                    action: MenuButtonAction::ChangeState(MenuState::Main),
                    "Back"
                }
            }
        }
    }
}

/// Credits menu dioxus component
#[component]
fn CreditsMenu(cx: Scope) -> Element {
    render! {
        MenuPanel {
            title: "Credits".to_string(),
            MenuPanelBody {
                "- bevy"
                "- bevy_dioxus"
                "- dioxus"
            },
            MenuPanelFooter {
                MenuButton {
                    action: MenuButtonAction::ChangeState(MenuState::Main),
                    "Back"
                }
            }
        }
    }
}

/// A menu panel centered in the middle of the screen
#[component]
fn MenuPanel<'a>(cx: Scope<'a, MenuPanelProps<'a>>) -> Element<'a> {
    render! {
        node {
            width: "100vw",
            height: "100vh",
            align_items: "center",
            justify_content: "center",
            node {
                flex_direction: "column",
                border_width: "1",
                border_color: "#fff",
                min_width: PANEL_MIN_WIDTH,
                // Header
                node {
                    padding: PANEL_PADDING,
                    align_items: "center",
                    border_width_bottom: "1",
                    border_color: "#fff",
                    "{cx.props.title}"
                }
                // Body
                &cx.props.children
            }
        }
    }
}

#[derive(Props)]
struct MenuPanelProps<'a> {
    title: String,
    children: Element<'a>,
}

/// Menu panel body component. Holds the main content in a menu panel.
#[component]
fn MenuPanelBody<'a>(cx: Scope<'a, ChildrenProps<'a>>) -> Element<'a> {
    render! {
        node {
            flex_direction: "column",
            padding: PANEL_PADDING,
            &cx.props.children
        }
    }
}

/// Menu panel footer component. Holds footer buttons at the bottom of a menu panel.
#[component]
fn MenuPanelFooter<'a>(cx: Scope<'a, ChildrenProps<'a>>) -> Element<'a> {
    render! {
        node {
            padding: PANEL_PADDING,
            flex_direction: "column",
            border_width_top: "1",
            border_color: "#fff",
            &cx.props.children
        }
    }
}

/// A button for menu navigation.
#[allow(non_snake_case)]
fn MenuButton<'a>(cx: Scope<'a, MenuButtonProps<'a>>) -> Element<'a> {
    let system_scheduler = use_system_scheduler(cx);
    let action = cx.props.action;

    render! {
        Button {
            margin_bottom: cx.props.margin_bottom,
            onclick: move |event: DioxusEvent<PointerButton>| if *event.data == PointerButton::Primary {
                system_scheduler.schedule({
                    move |world: &mut World| {
                        match action {
                            MenuButtonAction::ChangeState(state) => {
                                let mut next_state = world.resource_mut::<NextState<MenuState>>();
                                next_state.set(state);
                            }
                            MenuButtonAction::Exit => world.send_event(AppExit),
                        }
                    }
                });
                event.stop_propagation();
            },
            &cx.props.children
        }
    }
}

#[derive(Props)]
struct MenuButtonProps<'a> {
    action: MenuButtonAction,
    margin_bottom: Option<&'a str>,
    children: Element<'a>,
}

/// A standard button
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
            align_items: "center",
            justify_content: "center",
            margin_bottom: cx.props.margin_bottom.unwrap_or("0"),
            &cx.props.children
        }
    }
}

#[derive(Props)]
struct ButtonProps<'a> {
    onclick: EventHandler<'a, DioxusEvent<PointerButton>>,
    base_color: Option<&'a str>,
    click_color: Option<&'a str>,
    hover_color: Option<&'a str>,
    children: Element<'a>,
    margin_bottom: Option<&'a str>,
}
