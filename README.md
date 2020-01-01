# Experimental Rust implementation of Drawpile's paint engine

This is an experimental and a work-in-progress reimplementation of Drawpile's core. It may (or may not) one day end up in Drawpile as a replacement for the paint engine currently written in C++.
The primary purpose of this project is for me to learn Rust and to experiment with some new new architecture and optimization ideas for the paint engine.

Rewriting all of Drawpile is infeasible, but rewriting the important core parts does seem to be possible. This would have many advantages:

 * Improved code quality (hopefully...)
 * A library that implements Drawpile's core features (paint engine and protocol) could be used by multiple different client frontends: the Qt based desktop version, a WebAssembly version, and mobile versions using native toolkits.
 * The standalone server could be rewritten entirely in Rust for simpler deployment (no Qt dependency)
 * A pure Rust thick-server would be much more secure than the C++ based one
 * A fresh architecture designed to take advantage of multithreading.
 * Async/await would make the server code neater

A full reimplementation (with the GUI parts in C++) would be structured something like this:

 * (C++ app) drawpile-desktop: the Qt Widgets based  desktop application
 * (C++ app) drawpile-tabletpc: a future Qt Quick based implementation for tablet PCs
 * (C++ app) drawpile-server-gui: a Qt Widgets based graphical shell for drawpile-libserver
 * (C++ lib) drawpile-qt: the common GUI agnostic Qt parts of the client shared by the desktop and tabletpc versions
 * (Rust app) drawpile-cli: a reimplementation of drawpile-cmd headless recording playback tool. Maybe throw in dprectool functionaly as well.
 * (Rust app) drawpile-thin-server: the thin server (wraps drawpile-libserver)
 * (Rust app) drawpile-thick-server: the thick server (wraps drawpile-core and drawpile-libserver)
 * (Rust lib) drawpile-core: the paint engine and state tracker. (Everything needed by a headless drawpile instance, IO stuff excluded)
 * (Rust lib) drawpile-brushes: the brushes and other tools for use by GUI implementations?
 * (Rust lib) drawpile-libserver: the server implementation

Pie-in-the-sky stuff now within the realm of possibility:

 * Web app using a WebAssembly build of the libraries
 * Android and iOS clients using drawpile-core + drawpile-brushes

## Try it out!

Clone the repo and run `cargo test` to make sure everything works.

Run `cargo run --example` to get a list of available example programs.
E.g. running `cargo run --example layer_fillrect` will run the example program in `dpcore/examples/layer_fillrect.rs`. It writes out a file named `example_layer_fillrect.png` you can view.

## Current status

What is implemented:

 * Most of the paint engine

What's missing:

 * Parts of the paint engine:
   * Layer stack observer
   * Flood fill algorithm (not needed for headless mode)
 * State tracker
 * ACL filtering
 * Protocol (de)serialization

## License

This project is licensed under the GPLv3 with an App Store exception:

    As additional permission under section 7, you are allowed to distribute the
    software through an app store, even if that store has restrictive terms and
    conditions that are incompatible with the GPL, provided that the source is
    also available under the GPL with or without this permission through a
    channel without those restrictive terms and conditions.

