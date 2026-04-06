use std::rc::Rc;

use slint::{ComponentHandle as _, platform::Platform};
use slint_renderer_software_custom::{
    LineBufferProvider, MinimalSoftwareWindow, RepaintBufferType,
};

use crate::{MainWindow, lcd::Display, rgb565::BigEndianRgb565Pixel};

pub struct MyPlatform {
    window: Rc<MinimalSoftwareWindow>,
}

const WIDTH: usize = 320;
const HEIGHT: usize = 480;

impl Platform for MyPlatform {
    fn create_window_adapter(
        &self,
    ) -> Result<Rc<dyn slint::platform::WindowAdapter>, slint::PlatformError> {
        // Since on MCUs, there can be only one window, just return a clone of self.window.
        // We'll also use the same window in the event loop.
        Ok(self.window.clone())
    }
    fn duration_since_start(&self) -> core::time::Duration {
        core::time::Duration::from_micros(embassy_time::Instant::now().as_micros())
    }
    // optional: You can put the event loop there, or in the main function, see later
    fn run_event_loop(&self) -> Result<(), slint::PlatformError> {
        todo!();
    }
}

const NUM_BUFFERS: usize = 12;

struct LineBuf<'a> {
    display: &'a mut Display,
    buffers: &'a mut [[BigEndianRgb565Pixel; WIDTH]],
    buffer_idx: &'a mut usize,
}

impl<'a> LineBufferProvider for LineBuf<'a> {
    type TargetPixel = BigEndianRgb565Pixel;

    fn process_line(
        &mut self,
        line: usize,
        range: core::ops::Range<usize>,
        render_fn: impl FnOnce(&mut [Self::TargetPixel]),
    ) {
        let len = range.end - range.start;
        let buf = &mut self.buffers[*self.buffer_idx][0..len];
        render_fn(buf);

        let _ = self
            .display
            .draw(range.start as u16, line as u16, len as u16, 1, unsafe {
                core::slice::from_raw_parts(buf.as_ptr() as *const u8, len * 2)
            });
            
        *self.buffer_idx = (*self.buffer_idx + 1) % NUM_BUFFERS;
    }
}

#[allow(clippy::large_stack_frames)]
pub fn run_ui(display: &mut Display) {
    let window = MinimalSoftwareWindow::new(RepaintBufferType::ReusedBuffer);
    window.set_size(slint::PhysicalSize::new(WIDTH as u32, HEIGHT as u32));
    slint::platform::set_platform(Box::new(MyPlatform {
        window: window.clone(),
    }))
    .unwrap();

    let ui = MainWindow::new().unwrap();
    ui.show().expect("Failed to show the UI");

    let mut line_buffers = vec![[BigEndianRgb565Pixel(0, 0); WIDTH]; NUM_BUFFERS];
    let mut buffer_idx = 0;

    loop {
        // Let Slint run the timer hooks and update animatios.
        slint::platform::update_timers_and_animations();

        // Check the touch screen or input device using your driver.
        // while let Ok(e) = TOUCH_EVENT_CHAN.try_receive() {
        //     window.try_dispatch_event(e).unwrap();
        // }

        let frame_buffer = LineBuf {
            display,
            buffers: &mut line_buffers,
            buffer_idx: &mut buffer_idx,
        };

        // Draw the scene if something needs to be drawn.
        window.draw_if_needed(|renderer| {
            renderer.render_by_line(frame_buffer);
        });

        match (
            window.has_active_animations(),
            slint::platform::duration_until_next_timer_update(),
        ) {
            (true, _) => {}
            (false, Some(duration)) => {
                esp_idf_svc::hal::delay::FreeRtos::delay_ms(duration.as_millis() as u32);
            }
            (false, None) => {
                esp_idf_svc::hal::delay::FreeRtos::delay_ms(10);
            }
        }
    }
}
