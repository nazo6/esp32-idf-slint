use esp_idf_svc::hal::peripherals::Peripherals;

mod lcd;
mod platform;
mod rgb565;

slint::include_modules!();

fn main() {
    esp_idf_svc::sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();

    let peripherals = Peripherals::take().expect("Failed to take peripherals");

    log::info!("App start");

    let mut display = lcd::Display::new().expect("Failed to initialize LCD");
    platform::run_ui(&mut display);
}
