/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! Rust bindings for GDExtension, the extension API of [Godot](https://godotengine.org/) 4.
//!
//! This documentation is a work in progress.
//!
//! # Kinds of types
//!
//! Godot is written in C++, which doesn't have the same strict guarantees about safety and
//! mutability that Rust does. As a result, not everything in this crate will look and feel
//! entirely "Rusty". We distinguish four different kinds of types:
//!
//! 1. **Value types**: `i64`, `f64`, and mathematical types like
//!    [`Vector2`][crate::builtin::Vector2] and [`Color`][crate::builtin::Color].
//!
//!    These are the simplest to understand and to work with. They implement `Clone` and often
//!    `Copy` as well. They are implemented with the same memory layout as their counterparts in
//!    Godot itself, and typically have public fields. <br><br>
//!
//! 2. **Copy-on-write types**: [`GodotString`][crate::builtin::GodotString],
//!    [`StringName`][crate::builtin::StringName], and `Packed*Array` types.
//!
//!    These mostly act like value types, similar to Rust's own `Vec`. You can `Clone` them to get
//!    a full copy of the entire object, as you would expect.
//!
//!    Under the hood in Godot, these types are implemented with copy-on-write, so that data can be
//!    shared until one of the copies needs to be modified. However, this performance optimization
//!    is entirely hidden from the API and you don't normally need to worry about it. <br><br>
//!
//! 3. **Reference-counted types**: [`Array`][crate::builtin::Array],
//!    [`Dictionary`][crate::builtin::Dictionary], and [`Gd<T>`][crate::obj::Gd] where `T` inherits
//!    from [`RefCounted`][crate::engine::RefCounted].
//!
//!    These types may share their underlying data between multiple instances: changes to one
//!    instance are visible in another. Think of them as `Rc<RefCell<...>>` but without any runtime
//!    borrow checking.
//!
//!    Since there is no way to prevent or even detect this sharing from Rust, you need to be more
//!    careful when using such types. For example, when iterating over an `Array`, make sure that
//!    it isn't being modified at the same time through another reference.
//!
//!    To avoid confusion these types don't implement `Clone`. You can use
//!    [`Share`][crate::obj::Share] to create a new reference to the same instance, and
//!    type-specific methods such as
//!    [`Array::duplicate_deep()`][crate::builtin::Array::duplicate_deep] to make actual
//!    copies. <br><br>
//!
//! 4. **Manually managed types**: [`Gd<T>`][crate::obj::Gd] where `T` inherits from
//!    [`Object`][crate::engine::Object] but not from [`RefCounted`][crate::engine::RefCounted];
//!    most notably, this includes all `Node` classes.
//!
//!    These also share data, but do not use reference counting to manage their memory. Instead,
//!    you must either hand over ownership to Godot (e.g. by adding a node to the scene tree) or
//!    free them manually using [`Gd::free()`][crate::obj::Gd::free]. <br><br>
//!
//! # Ergonomics and panics
//!
//! The GDExtension Rust bindings are designed with usage ergonomics in mind, making them viable
//! for fast prototyping. Part of this design means that users should not constantly be forced
//! to write code such as `obj.cast::<T>().unwrap()`. Instead, they can just write `obj.cast::<T>()`,
//! which may panic at runtime.
//!
//! This approach has several advantages:
//! * The code is more concise and less cluttered.
//! * Methods like `cast()` provide very sophisticated panic messages when they fail (e.g. involved
//!   classes), immediately giving you the necessary context for debugging. This is certainly
//!   preferable over a generic `unwrap()`, and in most cases also over a `expect("literal")`.
//! * Usually, such methods panicking indicate bugs in the application. For example, you have a static
//!   scene tree, and you _know_ that a node of certain type and name exists. `get_node_as::<T>("name")`
//!   thus _must_ succeed, or your mental concept is wrong. In other words, there is not much you can
//!   do at runtime to recover from such errors anyway; the code needs to be fixed.
//!
//! Now, there are of course cases where you _do_ want to check certain assumptions dynamically.
//! Imagine a scene tree that is constructed at runtime, e.g. in a game editor.
//! This is why the library provides "overloads" for most of these methods that return `Option` or `Result`.
//! Such methods have more verbose names and highlight the attempt, e.g. `try_cast()`.
//!
//! To help you identify panicking methods, we use the symbol "⚠️" at the beginning of the documentation;
//! this should also appear immediately in the auto-completion of your IDE. Note that this warning sign is
//! not used as a general panic indicator, but particularly for methods which have a `Option`/`Result`-based
//! overload. If you want to know whether and how a method can panic, check if its documentation has a
//! _Panics_ section.
//!
//! # Thread safety
//!
//! [Godot's own thread safety
//! rules](https://docs.godotengine.org/en/latest/tutorials/performance/thread_safe_apis.html)
//! apply. Types in this crate implement (or don't implement) `Send` and `Sync` wherever
//! appropriate, but the Rust compiler cannot check what happens to an object through C++ or
//! GDScript.
//!
//! As a rule of thumb, if you must use threading, prefer to use Rust threads instead of Godot
//! threads.

#[doc(inline)]
pub use godot_core::{builtin, engine, log, obj, sys};

/// Facilities for initializing and terminating the GDExtension library.
pub mod init {
    pub use godot_core::init::*;

    // Re-exports
    pub use godot_macros::gdextension;
}

/// Export user-defined classes and methods to be called by the engine.
pub mod bind {

    // Re-exports
    pub use godot_macros::{godot_api, GodotClass};
}

/// Testing facilities (unstable).
#[doc(hidden)]
pub mod test {
    pub use godot_macros::itest;
}

#[doc(hidden)]
pub use godot_core::private;

/// Often-imported symbols.
pub mod prelude {
    pub use super::bind::{godot_api, GodotClass};
    pub use super::builtin::*;
    pub use super::builtin::{array, dict, varray}; // Re-export macros.
    pub use super::engine::{
        load, try_load, utilities, AudioStreamPlayer, AudioStreamPlayerVirtual, Camera2D,
        Camera2DVirtual, Camera3D, Camera3DVirtual, Input, Node, Node2D, Node2DVirtual, Node3D,
        Node3DVirtual, NodeVirtual, Object, ObjectVirtual, PackedScene, PackedSceneVirtual,
        RefCounted, RefCountedVirtual, Resource, ResourceVirtual, SceneTree, SceneTreeVirtual,
    };
    pub use super::init::{gdextension, ExtensionLayer, ExtensionLibrary, InitHandle, InitLevel};
    pub use super::log::*;
    pub use super::obj::{Base, Gd, GdMut, GdRef, GodotClass, Inherits, InstanceId, Share};

    // Make trait methods available
    pub use super::engine::NodeExt as _;
    pub use super::obj::EngineEnum as _;
}
