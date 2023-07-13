use clap::{arg, Command};

// https://github.com/clap-rs/clap/issues/4869
// 4.0 regression: dashes are not accepted any more #4869

pub fn set_flags() -> Command<'static> {
    let app = Command::new("waystream")
        .version(env!("CARGO_PKG_VERSION"))
        .author(env!("CARGO_PKG_AUTHORS"))
        .about("Streaming tool for Wayland compositors implementing zwlr_screencopy_v1")
        .arg(
            arg!(--debug)
                .required(false)
                .takes_value(false)
                .help("Enable debug mode"),
        )
        .arg(
            arg!(--slurp <GEOMETRY>)
                .required(false)
                .takes_value(true)
                .help("Select a portion of display to stream using slurp"),
        )
        .arg(
            arg!(--showfps <SHOW_FRAMERATE>)
                .required(false)
                .takes_value(false)
                .help("Show framerate"),
        )
        .arg(
            arg!(--cursor)
                .required(false)
                .takes_value(false)
                .help("Enable cursor in stream"),
        )
        .arg(
            arg!(--udphost <UDP_TARGET_HOST>)
                .required(true)
                .takes_value(true)
                .help("Set the host to stream to"),
        )
        .arg(
            arg!(--udpport <UDP_TARGET_PORT>)
                .required(true)
                .takes_value(true)
                .help("Set the port to stream to"),
        )
        .arg(
            arg!(--height <TARGET_HEIGHT>)
                .required(false)
                .takes_value(true)
                .help("Set the target video height"),
        )
        .arg(
            arg!(--width <TARGET_WIDTH>)
                .required(false)
                .takes_value(true)
                .help("Set the target video width"),
        )
        .arg(
            arg!(--listoutputs)
                .required(false)
                .takes_value(false)
                .help("List all outputs"),
        )
        .arg(
            arg!(--output <OUTPUT>)
                .required(false)
                .takes_value(true)
                .conflicts_with("slurp")
                .help("Select a particular display to stream (Not implemented yet)"),
        );
    app
}
