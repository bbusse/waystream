use clap::{arg, Command};
use std::iter::Map;

pub fn set_flags() -> Command {
    let app = Command::new("waystream")
        .version(env!("CARGO_PKG_VERSION"))
        .author(env!("CARGO_PKG_AUTHORS"))
        .about("Streaming tool for Wayland compositors implementing zwlr_screencopy_v1")
        .arg(
            arg!(--debug)
                .required(false)
                .num_args(0)
                .help("Enable debug mode"),
        )
        .arg(
            arg!(--slurp <GEOMETRY>)
                .required(false)
                .num_args(1)
                .help("Select a portion of display to stream using slurp"),
        )
        .arg(
            arg!(--"show-fps")
                .long("show-fps")
                .required(false)
                .num_args(0)
                .help("Show framerate"),
        )
        .arg(
            arg!(--"show-stream")
                .long("show-stream")
                .required(false)
                .num_args(0)
                .help("Show video output"),
        )
        .arg(
            arg!(--cursor)
                .required(false)
                .num_args(0)
                .help("Enable cursor in stream"),
        )
        .arg(
            arg!(--"http-host" <HTTP_HOST>)
                .long("http-host")
                .required(false)
                .num_args(1)
                .help("Set the http listen address")
        )
        .arg(
            arg!(--"http-port" <HTTP_PORT>)
                .long("http-port")
                .required(false)
                .num_args(1)
                .value_parser(clap::value_parser!(u16))
                .help("Set the http listen port"),
        )
        .arg(
            arg!(--"udp-host" <UDP_HOST> ...)
                .long("udp-host")
                .required(false)
                .num_args(1)
                .help("Set the host to stream to"),
        )
        .arg(
            arg!(--"udp-port" <UDP_PORT> ...)
                .long("udp-port")
                .required(false)
                .num_args(1)
                .value_parser(clap::value_parser!(i32))
                .help("Set the port to stream to"),
        )
        .arg(
            arg!(--height <TARGET_HEIGHT>)
                .required(false)
                .num_args(1)
                .value_parser(clap::value_parser!(i32))
                .help("Set the target video height"),
        )
        .arg(
            arg!(--width <TARGET_WIDTH>)
                .required(false)
                .num_args(1)
                .value_parser(clap::value_parser!(i32))
                .help("Set the target video width"),
        )
        .arg(
            arg!(--list-outputs)
                .required(false)
                .num_args(0)
                .help("List all outputs"),
        )
        .arg(
            arg!(--output <OUTPUT>)
                .required(false)
                .num_args(1)
                .conflicts_with("slurp")
                .help("Select a particular display to stream (Not implemented yet)"),
        );
    app
}
