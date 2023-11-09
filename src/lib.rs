use extism_pdk::*;
use lazy_static::lazy_static;
use std::sync::{Arc, Mutex};
use std::time::Instant;
use yaged::decoder;
use yaged::decoder::ColorOutput;
use yaged::types::Gif;

const GIF_DATA: &[u8] = include_bytes!("../assets/catjam.gif");

lazy_static! {
    static ref GIF: Arc<Mutex<Gif>> = Arc::new(Mutex::new(
        decoder::decode(GIF_DATA, ColorOutput::RGBA).unwrap()
    ));
    static ref N_FRAMES: Arc<Mutex<usize>> = Arc::new(Mutex::new(0));
    static ref CURR_FRAME_IDX: Arc<Mutex<usize>> = Arc::new(Mutex::new(0));
    static ref SRC_STATE: Arc<Mutex<Vec<Vec<[u8; 4]>>>> = Arc::new(Mutex::new(vec![]));
    static ref LAST_FRAME_TIME: Arc<Mutex<Instant>> = Arc::new(Mutex::new(Instant::now()));
}

#[plugin_fn]
pub fn setup(_: ()) -> FnResult<()> {
    // Set the number of frames
    let mut n_frames = N_FRAMES.lock().unwrap();
    *n_frames = GIF.lock().unwrap().frames().len();

    // Setup the LED state
    let gif = GIF.lock().unwrap();
    let (w, h) = (
        gif.screen_descriptor().width() as usize,
        gif.screen_descriptor().height() as usize,
    );
    let mut src_state = SRC_STATE.lock().unwrap();
    *src_state = vec![vec![[0; 4]; w]; h];

    Ok(())
}

#[plugin_fn]
pub fn update(_: ()) -> FnResult<Json<Vec<Vec<[u8; 4]>>>> {
    let width = config::get("width").unwrap().parse().unwrap();
    let height = config::get("height").unwrap().parse().unwrap();

    // Get the GIF data
    let gif = GIF.lock().unwrap();

    // Get references to the state variables
    let n_frames = N_FRAMES.lock().unwrap();
    let mut frame_idx = CURR_FRAME_IDX.lock().unwrap();
    let mut src_state = SRC_STATE.lock().unwrap();
    let mut last_frame_time = LAST_FRAME_TIME.lock().unwrap();

    // Make an empty array for the led state
    let mut led_state = vec![vec![[0u8; 4]; width]; height];

    // Reset the frame counter if we've gone over
    if *frame_idx >= *n_frames {
        *frame_idx = 0;
    }

    // Pull the current frame
    let current_frame = &gif.frames()[*frame_idx];

    // Pull the time between frames from the current frame
    let frame_delay_time: usize = match current_frame.graphic_control_extension() {
        None => { 0 }
        Some(graphic_control_ext) => {
            graphic_control_ext.delay_time() as usize // This is in 1/100ths of a second
        }
    };

    // Update the source state if a frame has passed
    let now = Instant::now();
    if (now - *last_frame_time).as_millis() >= (frame_delay_time * 10) as u128 {
        // Mark the last frame time
        *last_frame_time = now;

        // A note about what happens next:
        //
        // When processing a GIF frame by frame, most frames will not have data about the frame,
        // but instead as data about pixels within an sub-area of the entire frame. All pixels
        // within that sub-area are changed to the new value, and all pixels outside of that
        // sub-area are assumed to be unchanged from the last frame.
        //
        // This plugin holds the current state of the GIF, named SRC_STATE, which is updated every
        // frame as new information comes in. The code below will pull information about this
        // sub-area from the GIF decoder, and then update that sub-area within SRC_STATE. Later in
        // the code, the current, full-sized frame stored in SRC_STATE will be downscaled for the
        // dimensions of the LED matrix.

        // Pull the image descriptor from the frame
        let img_desc = current_frame.image_descriptor();

        // Pull the raster data
        let raster_data = current_frame.rgba_raster_data().clone().unwrap();

        // Mark the coordinates where the update starts and ends
        let (update_start_x, update_start_y) = (
            img_desc.image_left() as usize,
            img_desc.image_top() as usize,
        );
        let (update_end_x, update_end_y) = (
            update_start_x + img_desc.image_width() as usize,
            update_start_y + img_desc.image_height() as usize,
        );

        // Setup a variable pointing to the start of color data for the pixel we are currently processing
        let mut raster_data_idx = 0;

        // Update the source state
        for src_y in update_start_y..update_end_y {
            for src_x in update_start_x..update_end_x {
                // Only update if the alpha channel is present
                if raster_data[raster_data_idx + 3] != 0 {
                    // Populate the source state, in BGRA order
                    (*src_state)[src_y][src_x] = [
                        raster_data[raster_data_idx + 2], // b
                        raster_data[raster_data_idx + 1], // g
                        raster_data[raster_data_idx + 0], // r
                        raster_data[raster_data_idx + 3], // a
                    ];
                }

                // Increase index to point to the next pixel's color data
                raster_data_idx += 4;
            }
        }

        // Increase the frame pointer
        *frame_idx += 1;
    }

    // Calculate the image scaling
    let scale_x: f32 = gif.screen_descriptor().width() as f32 / width as f32;
    let scale_y: f32 = gif.screen_descriptor().height() as f32 / height as f32;

    // Use the source state to populate the led state
    for y in 0..height {
        for x in 0..width {
            // Use scalings to find pixel from source to sample from
            let (src_x, src_y) = (
                ((x as f32) * scale_x).floor() as usize,
                ((y as f32) * scale_y).floor() as usize,
            );
            led_state[y][x] = src_state[src_y][src_x];
        }
    }

    return Ok(Json(led_state));
}
