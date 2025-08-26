# Bevy Enhanced Input

[![crates.io](https://img.shields.io/crates/v/bevy_enhanced_input)](https://crates.io/crates/bevy_enhanced_input)
[![docs.rs](https://docs.rs/bevy_enhanced_input/badge.svg)](https://docs.rs/bevy_enhanced_input)
[![license](https://img.shields.io/crates/l/bevy_enhanced_input)](#license)
[![codecov](https://codecov.io/gh/simgine/bevy_enhanced_input/graph/badge.svg?token=wirFEuKmMz)](https://codecov.io/gh/simgine/bevy_enhanced_input)

Input manager for [Bevy](https://bevyengine.org), inspired by [Unreal Engine Enhanced Input](https://dev.epicgames.com/documentation/en-us/unreal-engine/enhanced-input-in-unreal-engine).

## Features

* Map inputs from various sources (keyboard, gamepad, etc.) to gameplay actions like `Jump`, `Move`, or `Attack`.
* Assign actions to different contexts like `OnFoot` or `InCar`.
* Layer multiple contexts on a single entity, controlled by priority.
* Apply modifiers to inputs, such as dead zones, inversion, scaling, etc., or create custom modifiers by implementing a trait.
* Assign conditions for how and when an action is triggered, like "hold", "tap", "chord", etc. You can also create custom conditions by implementing a trait.
* Control how actions accumulate input from sources and consume it.
* React to actions with observers or query them in systems.

## Getting Started

Check out the [quick start guide](https://docs.rs/bevy_enhanced_input) for more details.

See also examples in the repo. [simple_fly_cam.rs](examples/simple_fly_cam.rs) should be a good starting point.

Have any questions? Feel free to ask in the dedicated [`bevy_enhanced_input` channel](https://discord.com/channels/691052431525675048/1297361733886677036) in Bevy's Discord server.

## Bevy compatibility

| bevy        | bevy_enhanced_input |
| ----------- | ------------------- |
| 0.16.0      | 0.11-0.18           |
| 0.15.0      | 0.4-0.10            |
| 0.14.0      | 0.1-0.3             |

## License

Licensed under either of [Apache License, Version 2.0](LICENSE-APACHE) or [MIT License](LICENSE-MIT) at your option.
