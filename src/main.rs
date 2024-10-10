mod components;
mod consts;
mod events;
mod utils;

use components::*;
use consts::*;
use events::*;
use utils::*;

use bevy::{log::LogPlugin, prelude::*};
use bevy_kira_audio::prelude::*;
use rand::Rng;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins.set(LogPlugin {
                filter: "wgpu=error,naga=warn,bevy_gltf::loader=error,symphonia_bundle_mp3=error,symphonia_core=error"
                    .to_string(),
                ..default()
            }),
            bevy_inspector_egui::quick::WorldInspectorPlugin::new(),
            AudioPlugin,
        ))
        .register_type::<Pin>()
        .init_state::<LockpickState>()
        .insert_resource(PlayerLockpicks(100))
        .add_event::<TriggerPin>()
        .add_event::<CheckWin>()
        .add_event::<TryUnlockPin>()
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                trigger_pin,
                lock_pin,
                sync_lockpick_label,
                rise_pin,
                drop_pin.after(rise_pin),
                drop_pin_sound.after(rise_pin),
                //
                trigger_lockpick,
                up_lockpick.run_if(in_state(LockpickState::Up)),
                down_lockpick.run_if(in_state(LockpickState::Down)),
                //
                are_ya_winning_son,
                random_attemp,
            ),
        )
        .run();
}

#[derive(States, Component, Clone, Copy, PartialEq, Eq, Hash, Debug, Default)]
enum LockpickState {
    Up,
    Down,
    #[default]
    Unactive,
}

#[derive(Resource)]
struct PlayerLockpicks(u32);

fn random_attemp(
    mut commands: Commands,
    auto_attempt_button: Query<&Interaction, (Changed<Interaction>, With<AutoAttemptButton>)>,
    mut pins: Query<(Entity, &mut Style), With<Pin>>,
    mut springs: Query<&mut Style, (With<Spring>, Without<Pin>)>,
    mut check_win_writer: EventWriter<CheckWin>,
    mut player_lockpicks: ResMut<PlayerLockpicks>,
) {
    let Ok(interaction) = auto_attempt_button.get_single() else {
        return;
    };
    if *interaction != Interaction::Pressed {
        return;
    }

    if !rand::thread_rng().gen_bool(AUTO_ATTEMPT_CHANCE as f64) {
        player_lockpicks.0 -= 1;
        return;
    }

    for mut style in &mut springs {
        style.height = Val::Percent(0.0);
    }

    for (entity, mut style) in &mut pins {
        commands.entity(entity).insert(UnlockedPin);
        style.bottom = Val::Percent(MAX_PIN_BOTTOM_PERCENT);
    }

    check_win_writer.send_default();
}

fn lock_pin(
    asset_server: Res<AssetServer>,
    audio: Res<Audio>,
    mut commands: Commands,
    lockpick_state: Res<State<LockpickState>>,
    mut check_win_writer: EventWriter<CheckWin>,
    mut try_unlock_pin_reader: EventReader<TryUnlockPin>,
    unlocked_by_default_pins: Query<
        &Pin,
        (
            With<UnlockedByDefaultPin>,
            Without<TriggeredPin>,
            Without<DroppingPin>,
        ),
    >,
    mut active_pin: Query<
        (Entity, &mut Pin, &mut Style),
        (
            Or<(With<TriggeredPin>, With<DroppingPin>)>,
            Without<LockedPin>,
        ),
    >,
    locked_pins: Query<&Pin, (With<LockedPin>, Without<TriggeredPin>, Without<DroppingPin>)>,
    unlocked_pins: Query<
        (Entity, &Pin),
        (
            With<UnlockedPin>,
            Without<UnlockedByDefaultPin>,
            Without<TriggeredPin>,
            Without<DroppingPin>,
        ),
    >,
    mut player_lockpicks: ResMut<PlayerLockpicks>,
) {
    if lockpick_state.get().eq(&LockpickState::Unactive) {
        return;
    }

    if try_unlock_pin_reader.read().count() == 0 {
        return;
    }

    let Ok((entity, mut pin, mut style)) = active_pin.get_single_mut() else {
        return;
    };

    let bottom = val_as_percent(&style.bottom);

    if bottom <= 10.0 {
        return;
    }

    if bottom >= SWEET_SPOT_STARTS_AT {
        pin.rise_time = 0.0;
        commands
            .entity(entity)
            .insert(UnlockedPin)
            .remove::<DroppingPin>()
            .remove::<TriggeredPin>();
        style.bottom = Val::Percent(MAX_PIN_BOTTOM_PERCENT);
        check_win_writer.send_default();
        audio.play(asset_server.load("glasses-click.mp3"));
    } else {
        let excluded_pins = unlocked_by_default_pins
            .iter()
            .map(Pin::get_index)
            .chain(locked_pins.iter().map(Pin::get_index))
            .chain([pin.index])
            .collect::<Vec<_>>();
        let pins_to_drop = random_indexes(DROP_NUM_AFTER_FAIL, TUMBLERS, &excluded_pins);

        for (entity, pin) in &unlocked_pins {
            if pins_to_drop.contains(&pin.index) {
                commands
                    .entity(entity)
                    .insert(DroppingPin)
                    .remove::<UnlockedPin>();
            }
        }

        commands
            .entity(entity)
            .insert(DroppingPin)
            .remove::<TriggeredPin>();

        pin.rise_time = 0.0;
        pin.inc_next_rise();

        let mut rng = rand::thread_rng();

        if rng.gen_range(0.0..1.0) < LOCKPICK_LOSS_CHANCE {
            player_lockpicks.0 -= 1;

            audio.play(asset_server.load("lockpick-loss.mp3"));
        }
    }
}

fn are_ya_winning_son(
    audio: Res<Audio>,
    asset_server: Res<AssetServer>,
    unlocked_pins: Query<(), (With<UnlockedPin>, With<Pin>)>,
    mut check_win_reader: EventReader<CheckWin>,
    mut lock_bg: Query<&mut BackgroundColor, With<Lock>>,
) {
    if check_win_reader.read().count() == 0 {
        return;
    }

    if unlocked_pins.iter().count() == TUMBLERS {
        let mut lock_bg = lock_bg.single_mut();
        *lock_bg = Srgba::from_u8_array([222, 255, 146, 255]).into();
        audio.play(asset_server.load("happy-happy-happy.mp3"));
    }
}

fn rise_pin(
    time: Res<Time>,
    mut commands: Commands,
    mut triggered_pin: Query<(Entity, &mut Pin, &mut Style), With<TriggeredPin>>,
    mut triggered_pin_spring: Query<(&Spring, &mut Style), Without<Pin>>,
) {
    let Ok((entity, mut pin, mut style)) = triggered_pin.get_single_mut() else {
        return;
    };
    pin.rise_time += time.delta_seconds();

    let mut spring_style = triggered_pin_spring
        .iter_mut()
        .find_map(|(spring, style)| spring.0.eq(&pin.index).then_some(style))
        .unwrap();

    let progress = pin.get_progress();

    let mut next_bottom = TOTAL_PIN_CHANGE * progress;
    spring_style.height =
        Val::Percent(MAX_SPRING_HEIGHT_PERCENT - (TOTAL_SPRING_CHANGE * progress).max(0.0));

    if next_bottom >= MAX_PIN_BOTTOM_PERCENT && pin.is_time_limit_reached() {
        pin.rise_time = 0.0;
        pin.inc_next_rise();

        next_bottom = MAX_PIN_BOTTOM_PERCENT;
        commands
            .entity(entity)
            .insert(DroppingPin)
            .remove::<TriggeredPin>();
    }
    style.bottom = Val::Percent(next_bottom);
}

fn drop_pin_sound(
    audio: Res<Audio>,
    asset_server: Res<AssetServer>,
    dropping_pin: Query<(), Added<DroppingPin>>,
) {
    for _ in &dropping_pin {
        audio
            .play(asset_server.load("spring-vibration.mp3"))
            .with_volume(Volume::Amplitude(0.1));
    }
}

fn drop_pin(
    time: Res<Time>,
    mut commands: Commands,
    mut dropping_pin: Query<(Entity, &Pin, &mut Style), With<DroppingPin>>,
    mut triggered_pin_spring: Query<(&Spring, &mut Style), Without<Pin>>,
) {
    for (entity, pin, mut style) in &mut dropping_pin {
        let mut spring_style = triggered_pin_spring
            .iter_mut()
            .find_map(|(spring, style)| spring.0.eq(&pin.index).then_some(style))
            .unwrap();

        let bottom = val_as_percent(&style.bottom);
        let spring_height = val_as_percent(&spring_style.height);

        let shift = FALL_SHIFT_PER_MS * (time.delta().as_millis() as f32);
        let mut next_bottom = bottom - shift;
        spring_style.height = Val::Percent((spring_height + shift).min(45.0));

        if next_bottom <= MIN_PIN_BOTTOM_PERCENT {
            next_bottom = MIN_PIN_BOTTOM_PERCENT;
            commands
                .entity(entity)
                .remove::<DroppingPin>()
                .insert(LockedPin);
        }

        style.bottom = Val::Percent(next_bottom);
    }
}

fn trigger_pin(
    asset_server: Res<AssetServer>,
    audio: Res<Audio>,
    mut commands: Commands,
    mut trigger_pin_reader: EventReader<TriggerPin>,
    locked_pins: Query<(Entity, &Pin), Without<UnlockedPin>>,
) {
    for trigger_pin in trigger_pin_reader.read() {
        let pin_entity = locked_pins
            .iter()
            .find_map(|(entity, pin)| (pin.index == trigger_pin.0).then_some(entity));

        if let Some(pin_entity) = pin_entity {
            audio.play(asset_server.load("finger-snap.mp3"));

            commands
                .entity(pin_entity)
                .remove::<LockedPin>()
                .insert(TriggeredPin);
        }
    }
}

fn sync_lockpick_label(
    player_lockpicks: Res<PlayerLockpicks>,
    mut lockpick_label_query: Query<&mut Text, With<LockpickLabel>>,
) {
    if player_lockpicks.is_changed() {
        let mut label = lockpick_label_query.single_mut();
        *label = Text::from_section(
            format!("Lockpicks: {}", player_lockpicks.0),
            TextStyle {
                font_size: 32.0,
                ..default()
            },
        );
    }
}

fn up_lockpick(
    time: Res<Time>,
    mut q: Query<&mut Style, With<Lockpick>>,
    mut next_lockpick_state: ResMut<NextState<LockpickState>>,
) {
    let mut lockpick_style = q.single_mut();

    let bottom = val_as_px(&lockpick_style.bottom);

    let mut next_bottom = bottom + (LOCKPICK_SPEED_PER_MS * time.delta().as_millis() as f32);

    if next_bottom >= MAX_LOCKPICK_BOTTOM_PERCENT {
        next_bottom = MAX_LOCKPICK_BOTTOM_PERCENT;
        next_lockpick_state.set(LockpickState::Down);
    }

    lockpick_style.bottom = Val::Px(next_bottom);
}

fn down_lockpick(
    time: Res<Time>,
    mut q: Query<&mut Style, With<Lockpick>>,
    mut next_lockpick_state: ResMut<NextState<LockpickState>>,
) {
    let mut lockpick_style = q.single_mut();

    let bottom = val_as_px(&lockpick_style.bottom);

    let mut next_bottom = bottom - (LOCKPICK_SPEED_PER_MS * time.delta().as_millis() as f32);

    if next_bottom <= MIN_LOCKPICK_BOTTOM_PERCENT {
        next_bottom = MIN_LOCKPICK_BOTTOM_PERCENT;
        next_lockpick_state.set(LockpickState::Unactive);
    }

    lockpick_style.bottom = Val::Px(next_bottom);
}

fn trigger_lockpick(
    keys: Res<ButtonInput<KeyCode>>,
    mut next_lockpick_state: ResMut<NextState<LockpickState>>,
    mut q: Query<(&mut Style, &mut Lockpick)>,
    mut trigger_pin_writer: EventWriter<TriggerPin>,
    mut try_unlock_pin_writer: EventWriter<TryUnlockPin>,
    active_pin: Query<&Pin, Or<(With<TriggeredPin>, With<DroppingPin>)>>,
) {
    let (mut lockpick_style, mut lockpick) = q.single_mut();

    if keys.just_pressed(KeyCode::ArrowUp) {
        next_lockpick_state.set(LockpickState::Up);

        if active_pin.is_empty() {
            trigger_pin_writer.send(TriggerPin(lockpick.current_position));
        } else {
            try_unlock_pin_writer.send_default();
        }
    }

    if !active_pin.is_empty() {
        return;
    }

    if keys.just_pressed(KeyCode::ArrowLeft) {
        lockpick.current_position = lockpick.current_position.checked_sub(1).unwrap_or(0);

        lockpick_style.left = Val::Px(LOCKPICK_POSITIONS[lockpick.current_position as usize]);
    }

    if keys.just_pressed(KeyCode::ArrowRight) {
        lockpick.current_position = (lockpick.current_position + 1).min(5);

        lockpick_style.left = Val::Px(LOCKPICK_POSITIONS[lockpick.current_position as usize]);
    }
}

fn setup(
    mut commands: Commands,
    player_lockpicks: Res<PlayerLockpicks>,
    asset_server: Res<AssetServer>,
) {
    commands.spawn(Camera2dBundle::default());

    let unlocked_by_default = random_indexes(UNLOCKED_BY_DEFAULT_NUM, TUMBLERS, &[]);

    commands.spawn((
        Name::new("Lockpicks"),
        LockpickLabel,
        TextBundle {
            background_color: Color::BLACK.into(),
            style: Style {
                margin: UiRect::all(Val::Px(20.0)),
                ..default()
            },
            text: Text::from_section(
                format!("Lockpicks: {}", player_lockpicks.0),
                TextStyle {
                    font_size: 32.0,
                    ..default()
                },
            ),
            ..default()
        },
    ));

    commands
        .spawn((
            AutoAttemptButton,
            Name::new("Auto Attempt Button"),
            ButtonBundle {
                border_radius: BorderRadius::all(Val::Px(10.0)),
                border_color: Color::BLACK.into(),
                background_color: Srgba::from_u8_array([20, 20, 20, 255]).into(),
                style: Style {
                    align_self: AlignSelf::Center,
                    margin: UiRect::left(Val::Px(20.0)),
                    padding: UiRect::all(Val::Px(10.0)),
                    border: UiRect::all(Val::Px(5.0)),
                    ..default()
                },
                ..default()
            },
        ))
        .with_children(|parent| {
            parent.spawn(TextBundle::from_section(
                "Auto Attempt",
                TextStyle {
                    font_size: 40.0,
                    color: Color::WHITE.into(),
                    ..default()
                },
            ));
        });

    commands
        .spawn((
            Name::new("Lock"),
            Lock,
            NodeBundle {
                style: Style {
                    height: Val::Px(600.0),
                    width: Val::Px(600.0),
                    justify_content: JustifyContent::SpaceAround,
                    align_items: AlignItems::FlexStart,
                    align_self: AlignSelf::Center,
                    justify_self: JustifySelf::Center,
                    ..default()
                },
                background_color: Srgba::rgb_u8(230, 230, 230).into(),
                ..default()
            },
        ))
        .with_children(|parent| {
            parent
                .spawn((
                    Name::new("Tumblers"),
                    NodeBundle {
                        style: Style {
                            display: Display::Flex,
                            justify_content: JustifyContent::SpaceAround,
                            align_items: AlignItems::FlexStart,
                            height: Val::Px(230.0),
                            margin: UiRect::all(Val::Px(30.0)),
                            padding: UiRect::all(Val::Px(20.0)),
                            column_gap: Val::Px(20.0),
                            align_self: AlignSelf::Center,
                            justify_self: JustifySelf::Center,
                            ..default()
                        },
                        border_radius: BorderRadius::all(Val::Px(20.0)),
                        background_color: Srgba::rgba_u8(113, 113, 113, 90).into(),
                        ..default()
                    },
                ))
                .with_children(|parent| {
                    for i in 0..TUMBLERS {
                        parent
                            .spawn((
                                Name::new(format!("Tumbler #{i}")),
                                ImageBundle {
                                    style: Style {
                                        overflow: Overflow::clip(),
                                        display: Display::Flex,
                                        width: Val::Px(36.0),
                                        height: Val::Px(183.0),
                                        position_type: PositionType::Relative,
                                        justify_content: JustifyContent::Center,
                                        ..default()
                                    },
                                    image: UiImage::new(asset_server.load("Tumbler.png")),
                                    ..default()
                                },
                            ))
                            .with_children(|parent| {
                                if !unlocked_by_default.contains(&i) {
                                    parent.spawn((
                                        Name::new(format!("Spring #{i}")),
                                        Spring(i),
                                        ImageBundle {
                                            style: Style {
                                                width: Val::Px(34.0),
                                                height: Val::Percent(MAX_SPRING_HEIGHT_PERCENT),
                                                position_type: PositionType::Absolute,
                                                top: Val::Percent(0.0),
                                                ..default()
                                            },
                                            image: UiImage::new(asset_server.load("PinSpring.png")),
                                            ..default()
                                        },
                                    ));
                                }
                            })
                            .with_children(|parent| {
                                let mut node_bundle = ImageBundle {
                                    style: Style {
                                        width: Val::Px(34.0),
                                        height: Val::Px(109.0),
                                        position_type: PositionType::Absolute,
                                        bottom: Val::Percent(0.0),
                                        ..default()
                                    },
                                    image: UiImage::new(asset_server.load("Pin.png")),
                                    ..default()
                                };

                                let mut pin =
                                    parent.spawn((Name::new(format!("Pin #{i}")), Pin::new(i)));

                                if unlocked_by_default.contains(&i) {
                                    node_bundle.style.bottom = Val::Percent(MAX_PIN_BOTTOM_PERCENT);
                                    pin.insert((UnlockedByDefaultPin, UnlockedPin, node_bundle));
                                } else {
                                    pin.insert((LockedPin, node_bundle));
                                }
                            });
                    }
                });
        })
        .with_children(|parent| {
            parent.spawn((
                Name::new("Lockpick"),
                Lockpick {
                    current_position: 0,
                },
                ImageBundle {
                    style: Style {
                        position_type: PositionType::Absolute,
                        left: Val::Px(LOCKPICK_POSITIONS[0]),
                        bottom: Val::Px(MIN_LOCKPICK_BOTTOM_PERCENT),
                        ..default()
                    },
                    image: UiImage::new(asset_server.load("Lockpick.png")),
                    ..default()
                },
            ));
        });
}
