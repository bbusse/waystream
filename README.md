# waystream - Wayland Desktop Streaming
waystream streams your Wayland desktop session over the network  
  
It was originally forked from [wayshot](https://github.com/waycrate/wayshot) - a screenshot tool for Wayland
and has since been turned into a Wayland desktop streaming application


> **Note**
> Note that it is currently limited to compositors implementing zwlr_screencopy_v1

## Usage
Stream raw video to some host over UDP, show framerate data as overlay
```
$ waystream --udphost 127.0.0.1 --udpport 2342 --showfps
```
Stream raw video to some host over UDP, show framerate data as overlay,  
scale video to width and height
```
$ waystream --udphost 127.0.0.1 --udpport 2342 --showfps --width 320 --height 240
```
Show usage
```
$ waystream --help

waystream 0.1.0
Streaming tool for Wayland compositors implementing zwlr_screencopy_v1.

USAGE:
    waystream [OPTIONS] --udphost <UDP_TARGET_HOST> --udpport <UDP_TARGET_PORT>

OPTIONS:
    -c, --cursor                       Enable cursor in stream
    -d, --debug                        Enable debug mode
    -h, --udphost <UDP_TARGET_HOST>    Set the host to stream to
    -h, --udpport <UDP_TARGET_PORT>    Set the port to stream to
        --help                         Print help information
    -l, --listoutputs                  List all valid outputs
    -o, --output <OUTPUT>              Choose a particular display to stream
    -r, --showfps                      Show framerate
    -s, --slurp <GEOMETRY>             Select a portion of display to stream using slurp
        --stdout                       Output the image data to standard out
    -V, --version                      Print version information
    -x, --width <TARGET_WIDTH>         Set the target video width
    -y, --height <TARGET_HEIGHT>       Set the target video height
```
## Build
### Install build dependencies
```
$ sudo apt install cargo \
                   libgstreamer1.0-dev \
                   libgstreamer-plugins-base1.0-dev \
                   libglib2.0-dev \
                   libunwind-dev
```
### Build
```
$ cargo build --release
```
#### Run build
```
$ ./target/release/waystream
```

## Debug
### Profile
```
$ perf record --call-graph dwarf,16384 -e cpu-clock -F 997 ./target/release/waystream --udphost 127.0.0.1 --udpport 2342
$ perf script | stackcollapse-perf.pl | ./rust-unmangle | flamegraph.pl > flame.svg
```
## Authors
Björn Busse <bj.rn@baerlin.eu>  
Wladimir Leuschner <https://github.com/wleuschner>  
Shinyzenith <aakashsensharma@gmail.com> (wayshot/libwayshot)  
  
Special thanks to hrmny!
## Contributors

## References
[wayshot](https://github.com/waycrate/wayshot)  
[Wayland Protocol](https://wayland.freedesktop.org/)  
[Wayland Protocol in Wikipedia](https://en.wikipedia.org/wiki/Wayland_(protocol))  
[wlroots compositor](https://gitlab.freedesktop.org/wlroots/wlroots)  
