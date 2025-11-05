// Copyright 2025 the UI Events Authors
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Support routines for converting pointer data from [`winit`].

use ui_events::pointer::PointerButton;
use winit::event::MouseButton;

/// Try to make a [`PointerButton`] from a [`MouseButton`].
///
/// Because values of [`MouseButton::Other`] can start at 0, they are mapped
/// to the arbitrary buttons B7..B32.
/// Values greater than 25 will not be mapped.
pub fn try_from_winit_button(b: MouseButton) -> Option<PointerButton> {
    Some(match b {
        MouseButton::Left => PointerButton::Primary,
        MouseButton::Right => PointerButton::Secondary,
        MouseButton::Middle => PointerButton::Auxiliary,
        MouseButton::Back => PointerButton::X1,
        MouseButton::Forward => PointerButton::X2,
        MouseButton::Button6 => todo!(),
        MouseButton::Button7 => todo!(),
        MouseButton::Button8 => todo!(),
        MouseButton::Button9 => todo!(),
        MouseButton::Button10 => todo!(),
        MouseButton::Button11 => todo!(),
        MouseButton::Button12 => todo!(),
        MouseButton::Button13 => todo!(),
        MouseButton::Button14 => todo!(),
        MouseButton::Button15 => todo!(),
        MouseButton::Button16 => todo!(),
        MouseButton::Button17 => todo!(),
        MouseButton::Button18 => todo!(),
        MouseButton::Button19 => todo!(),
        MouseButton::Button20 => todo!(),
        MouseButton::Button21 => todo!(),
        MouseButton::Button22 => todo!(),
        MouseButton::Button23 => todo!(),
        MouseButton::Button24 => todo!(),
        MouseButton::Button25 => todo!(),
        MouseButton::Button26 => todo!(),
        MouseButton::Button27 => todo!(),
        MouseButton::Button28 => todo!(),
        MouseButton::Button29 => todo!(),
        MouseButton::Button30 => todo!(),
        MouseButton::Button31 => todo!(),
        MouseButton::Button32 => todo!(),
    })
}
