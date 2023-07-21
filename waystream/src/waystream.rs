use std::{
    cmp, env,
    error::Error,
    net::{IpAddr,Ipv6Addr},
    process::exit,
    time::{SystemTime},
};

use libwayshot::CaptureRegion;
use wayland_client::{
    globals::{registry_queue_init, GlobalList, GlobalListContents},
    protocol::{wl_output::WlOutput, wl_registry},
    Connection, QueueHandle,
};

use anyhow::Error as aError;
use derive_more::{Display, Error};
use gstreamer::glib;
use gstreamer::prelude::*;
use gstreamer_video;

#[derive(Debug, Display, Error)]
#[display(fmt = "Missing element {}", _0)]
struct MissingElement(#[error(not(source))] &'static str);

#[derive(Debug, Display, Error)]
#[display(fmt = "Received error from {}: {} (debug: {:?})", src, error, debug)]
struct ErrorMessage {
    src: glib::GString,
    error: glib::Error,
    debug: Option<glib::GString>,
}

struct PipeOptions {
    width: usize,
    height: usize,
    target_width: i32,
    target_height: i32,
    show_fps: bool,
    show_stream: bool,
    udp_host: String,
    udp_port: i32,
    http_host: String,
    http_port: u16,
}

mod clap;
mod output;

// TODO: Create a xdg-shell surface, check for the enter event, grab the output from it.

struct WaystreamState {}

impl wayland_client::Dispatch<wl_registry::WlRegistry, GlobalListContents> for WaystreamState {
    fn event(
        _: &mut WaystreamState,
        _: &wl_registry::WlRegistry,
        _: wl_registry::Event,
        _: &GlobalListContents,
        _: &Connection,
        _: &QueueHandle<WaystreamState>,
    ) {
    }
}

struct IntersectingOutput {
    output: WlOutput,
    region: CaptureRegion,
}
#[derive(Clone)]
#[derive(Debug)]
enum CaptureInfo {
    Region(CaptureRegion),
    Output(WlOutput),
}

fn parse_geometry(g: &str) -> Option<libwayshot::CaptureRegion> {
    let tail = g.trim();
    let x_coordinate: i32;
    let y_coordinate: i32;
    let width: i32;
    let height: i32;

    if tail.contains(',') {
        // this accepts: "%d,%d %dx%d"
        let (head, tail) = tail.split_once(',')?;
        x_coordinate = head.parse::<i32>().ok()?;
        let (head, tail) = tail.split_once(' ')?;
        y_coordinate = head.parse::<i32>().ok()?;
        let (head, tail) = tail.split_once('x')?;
        width = head.parse::<i32>().ok()?;
        height = tail.parse::<i32>().ok()?;
    } else {
        // this accepts: "%d %d %d %d"
        let (head, tail) = tail.split_once(' ')?;
        x_coordinate = head.parse::<i32>().ok()?;
        let (head, tail) = tail.split_once(' ')?;
        y_coordinate = head.parse::<i32>().ok()?;
        let (head, tail) = tail.split_once(' ')?;
        width = head.parse::<i32>().ok()?;
        height = tail.parse::<i32>().ok()?;
    }

    Some(libwayshot::CaptureRegion {
        x_coordinate,
        y_coordinate,
        width,
        height,
    })
}

fn create_pipeline(mut conn: Connection,
                   mut globals: GlobalList,
                   area: CaptureInfo,
                   pipe_opts: PipeOptions,
                   cursor_overlay: i32) -> Result<gstreamer::Pipeline, aError> {

    gstreamer::init()?;

    let pipeline = gstreamer::Pipeline::default();

    let video_info = gstreamer_video::VideoInfo::builder(gstreamer_video::VideoFormat::Rgbx, pipe_opts.width as u32, pipe_opts.height as u32)
            //.fps(gstreamer::Fraction::new(25, 1))
            .build()
            .expect("Failed to create video info");

    let appsrc = gstreamer_app::AppSrc::builder()
        .caps(&video_info.to_caps().unwrap())
        .format(gstreamer::Format::Time)
        .build();

    // Convert for each sink
    let videoconvert_0 = gstreamer::ElementFactory::make("videoconvert").build()?;
    let videoconvert_1 = gstreamer::ElementFactory::make("videoconvert").build()?;

    let videosink = gstreamer::ElementFactory::make("waylandsink").build()?;
    let fpssink = gstreamer::ElementFactory::make("fpsdisplaysink").property("video-sink", &videosink)
                                                                   .property("text-overlay", pipe_opts.show_fps)
                                                                   .build()?;

    let x264enc = gstreamer::ElementFactory::make("x264enc").build()
                                                            .expect("Failed to instantiate x264enc");

    let h264parse = gstreamer::ElementFactory::make("h264parse").build()
                                                                 .expect("Failed to instantiate h264parse");

    let scale = gstreamer::ElementFactory::make("videoscale")
        .name("scale")
        .build()
        .expect("Could not create convert element");

    let video_caps_scale = gstreamer::Caps::builder("video/x-raw")
        .field("width", pipe_opts.target_width)
        .field("height", pipe_opts.target_height)
        .build();

    let filter = gstreamer::ElementFactory::make("capsfilter")
        .name("caps")
        .build()
        .expect("Could not create caps element");

    if pipe_opts.target_height > 0 && pipe_opts.target_height > 0 {
        filter.set_property("caps", &video_caps_scale);
    }

    let audio_tee = gstreamer::ElementFactory::make("tee")
        .property("allow-not-linked", true)
        .build()?;

    let video_tee = gstreamer::ElementFactory::make("tee")
        .property("allow-not-linked", true)
        .build()?;

    let video_tee_queue_local = gstreamer::ElementFactory::make("queue").build()?;

    pipeline.add_many(&[appsrc.upcast_ref(), &scale,
                                             &filter,
                                             &video_tee,
                                             &video_tee_queue_local])?;

    gstreamer::Element::link_many(&[appsrc.upcast_ref(), &scale, &filter, &video_tee])?;

    if pipe_opts.show_stream {
        log::info!("Adding local video sink");
        pipeline.add(&fpssink).unwrap();
    }

    if let Ok(http_address) = pipe_opts.http_host.parse::<IpAddr>() {
        log::info!("Adding hlssink");

        let video_tee_queue_hls = gstreamer::ElementFactory::make("queue") .build()?;
        let hlssink = gstreamer::ElementFactory::make("hlssink2")
            .name("hlssink")
            .property("target-duration", 2u32)
            .property("playlist-length", 2u32)
            .property("max-files", 2u32)
            .build()
            .expect("Failed to instantiate hlssink");

        pipeline.add(&video_tee_queue_hls).unwrap();
        pipeline.add(&hlssink).unwrap();

        //gstreamer::Element::link_many(&[&video_tee, &video_tee_queue_hls, &hlssink])?;
    }

    if let Ok(udp_address) = pipe_opts.udp_host.parse::<IpAddr>() {
        log::info!("Adding udp video sink");
        let netsink = gstreamer::ElementFactory::make("udpsink").property("host", udp_address.to_string())
                                                                .property("port", pipe_opts.udp_port)
                                                                .build()?;

        let video_tee_queue_udp = gstreamer::ElementFactory::make("queue").build()?;

        pipeline.add(&video_tee_queue_udp).unwrap();
        pipeline.add(&netsink).unwrap();

        gstreamer::Element::link_many(&[&video_tee, &video_tee_queue_udp, &netsink])?;
    }

    if pipe_opts.show_stream {
        gstreamer::Element::link_many(&[&video_tee, &video_tee_queue_local, &fpssink])?;
    }

    let mut current_frame = 0;

    appsrc.set_callbacks(
        gstreamer_app::AppSrcCallbacks::builder()
            .need_data(move |appsrc, _| {
                log::debug!("Frame {current_frame}");
                let t0 = SystemTime::now();

                let frame_copy: (Vec<libwayshot::FrameCopy>, Option<(i32, i32)>) = match &area {
                    CaptureInfo::Region(region) => {
                        let mut framecopys = Vec::new();

                        let outputs = output::get_all_outputs(&mut globals, &mut conn);
                        let mut intersecting_outputs: Vec<IntersectingOutput> = Vec::new();
                        for output in outputs.iter() {
                            let x1: i32 = cmp::max(output.dimensions.x, region.x_coordinate);
                            let y1: i32 = cmp::max(output.dimensions.y, region.y_coordinate);
                            let x2: i32 = cmp::min(
                                output.dimensions.x + output.dimensions.width,
                                region.x_coordinate + region.width,
                            );
                            let y2: i32 = cmp::min(
                                output.dimensions.y + output.dimensions.height,
                                region.y_coordinate + region.height,
                            );

                            let width = x2 - x1;
                            let height = y2 - y1;

                            if !(width <= 0 || height <= 0) {
                                let true_x = region.x_coordinate - output.dimensions.x;
                                let true_y = region.y_coordinate - output.dimensions.y;
                                let true_region = CaptureRegion {
                                    x_coordinate: true_x,
                                    y_coordinate: true_y,
                                    width: region.width,
                                    height: region.height,
                                };
                                intersecting_outputs.push(IntersectingOutput {
                                    output: output.wl_output.clone(),
                                    region: true_region,
                                });
                            }
                        }
                        if intersecting_outputs.is_empty() {
                            log::error!("Provided capture region doesn't intersect with any outputs!");
                            exit(1);
                        }

                        for ouput_info in intersecting_outputs {
                            framecopys.push(libwayshot::capture_output_frame(
                                &mut globals,
                                &mut conn,
                                cursor_overlay,
                                ouput_info.output.clone(),
                                Some(ouput_info.region),
                            ).unwrap());
                        }
                        (framecopys, Some((region.width, region.height)))
                    }
                    CaptureInfo::Output(output) => (
                        vec![libwayshot::capture_output_frame(
                            &mut globals,
                            &mut conn,
                            cursor_overlay,
                            output.to_owned(),
                            None,
                        ).unwrap()],
                        None,
                    ),
                };

                // Create the buffer that can hold exactly one RGBx/BGRx frame
                let mut buffer = gstreamer::Buffer::with_size(video_info.size()).unwrap();
                {
                    let buffer = buffer.get_mut().unwrap();
                    buffer.set_pts(current_frame * 20 * gstreamer::ClockTime::MSECOND);

                    let mut vframe =
                        gstreamer_video::VideoFrameRef::from_buffer_ref_writable(buffer, &video_info)
                            .unwrap();

                    let width = vframe.width() as usize;
                    let height = vframe.height() as usize;
                    let stride = vframe.plane_stride()[0] as usize;

                    let mut npixel: usize = 0;

                    for line in vframe
                        .plane_data_mut(0)
                        .unwrap()
                        .chunks_exact_mut(stride)
                        .take(height)
                    {
                        for pixel in line[..(4 * width)].chunks_exact_mut(4) {
                            pixel[0] = frame_copy.0[0].frame_mmap[npixel];
                            pixel[1] = frame_copy.0[0].frame_mmap[npixel+1];
                            pixel[2] = frame_copy.0[0].frame_mmap[npixel+2];
                            pixel[3] = frame_copy.0[0].frame_mmap[npixel+3];
                            npixel += 4
                        }
                    }
                }
                current_frame += 1;
                log::debug!("{:?}", t0.elapsed());
                let _ = appsrc.push_buffer(buffer);
            })
            .build(),
    );
    Ok(pipeline)
}

fn stream(pipeline: gstreamer::Pipeline) -> Result<(), aError> {
    pipeline.set_state(gstreamer::State::Playing)?;

    let bus = pipeline
        .bus()
        .expect("Pipeline without bus. Shouldn't happen!");

    for msg in bus.iter_timed(gstreamer::ClockTime::NONE) {
        use gstreamer::MessageView;

        match msg.view() {
            MessageView::Eos(..) => break,
            MessageView::Error(err) => {
                pipeline.set_state(gstreamer::State::Null)?;
                return Err(ErrorMessage {
                    src: msg
                        .src()
                        .map(|s| s.path_string())
                        .unwrap_or_else(|| glib::GString::from("UNKNOWN")),
                    error: err.error(),
                    debug: err.debug(),
                }
                .into());
            }
            _ => (),
        }
    }

    pipeline.set_state(gstreamer::State::Null)?;

    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = clap::set_flags().get_matches();
    env::set_var("RUST_LOG", "waystream=info");

    if args.get_flag("debug") {
        env::set_var("RUST_LOG", "waystream=trace");
    }

    let mut pipe_opts = PipeOptions {
        width: 1366,
        height: 768,
        target_width: 0,
        target_height: 0,
        show_fps: false,
        show_stream: false,
        http_host: "".to_string(),
        http_port: 0,
        udp_host: "".to_string(),
        udp_port: 0,
    };

    if args.get_flag("show-fps") {
        pipe_opts.show_fps = true;
    }

    if args.get_flag("show-stream") {
        pipe_opts.show_stream = true;
    }

    if args.contains_id("http-host") {
        pipe_opts.http_host = args.get_one::<String>("http-host").unwrap().clone();
    }

    if args.contains_id("http-port") {
        pipe_opts.http_port = args.get_one::<u16>("http-port").unwrap().clone();
    }

    if args.contains_id("udp-host") {
        pipe_opts.udp_host = args.get_one::<String>("udp-host").unwrap().clone();
    }

    if args.contains_id("udp-port") {
        pipe_opts.udp_port = args.get_one::<i32>("udp-port").unwrap().clone();
    }

    if args.contains_id("width") {
        pipe_opts.target_width = args.get_one::<i32>("width").unwrap().clone();
    }

    if args.contains_id("height") {
        pipe_opts.target_height = args.get_one::<i32>("height").unwrap().clone();
    }

    env_logger::init();

    let mut conn = Connection::connect_to_env().unwrap();
    let (mut globals, _) = registry_queue_init::<WaystreamState>(&conn).unwrap();

    if args.contains_id("list-outputs") {
        let valid_outputs = output::get_all_outputs(&mut globals, &mut conn);
        for output in valid_outputs {
            log::info!("{:#?}", output.name);
        }
        exit(1);
    }

    let output: WlOutput = if args.contains_id("output") {
        output::get_wloutput(
            args.get_one::<String>("output").unwrap().clone(),
            output::get_all_outputs(&mut globals, &mut conn),
        )
    } else { output::get_all_outputs(&mut globals, &mut conn) .first()
            .unwrap()
            .wl_output
            .clone()
    };

    let mut cursor_overlay: i32 = 0;
    if args.get_flag("cursor") {
        cursor_overlay = 1;
    }

    let capture_area = {
        let mut start_x = 0;
        let mut start_y = 0;

        let mut end_x = 0;
        let mut end_y = 0;

        let output_infos = output::get_all_outputs(&mut globals, &mut conn);
        for outputinfo in output_infos {
            if outputinfo.dimensions.x < start_x {
                start_x = outputinfo.dimensions.x;
            }
            if outputinfo.dimensions.y < start_y {
                start_y = outputinfo.dimensions.y;
            }
            if outputinfo.dimensions.x + outputinfo.dimensions.width > end_x {
                end_x = outputinfo.dimensions.x + outputinfo.dimensions.width;
            }
            if outputinfo.dimensions.y + outputinfo.dimensions.height > end_y {
                end_y = outputinfo.dimensions.y + outputinfo.dimensions.height;
            }
        }
        CaptureInfo::Region(CaptureRegion {
            x_coordinate: start_x,
            y_coordinate: start_y,
            width: end_x - start_x,
            height: end_y - start_y,
        })
    };

    if let CaptureInfo::Region(r) = capture_area {
        pipe_opts.height = usize::try_from(r.height).unwrap();
        pipe_opts.width = usize::try_from(r.width).unwrap();
    };


    match create_pipeline(conn,
                          globals,
                          capture_area,
                          pipe_opts,
                          cursor_overlay).and_then(stream) {
        Ok(r) => r,
        Err(e) => eprintln!("Error running pipeline: {e}"),
    }

    Ok(())
}
