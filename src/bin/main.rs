#![no_std]
#![no_main]
#![deny(
    clippy::mem_forget,
    reason = "mem::forget is generally not safe to do with esp_hal types, especially those \
    holding buffers for the duration of a data transfer."
)]
#![deny(clippy::large_stack_frames)]

use esp_hal::clock::CpuClock;
use esp_hal::delay::Delay;
use esp_hal::gpio::{Level, Output, OutputConfig, Input, InputConfig, Pull};
use esp_hal::main;
use esp_println::println;
use esp_hal::time::Instant;  

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    println!("PANIC: {}", info);
    loop {}
}

// This creates a default app-descriptor required by the esp-idf bootloader.
// For more information see: <https://docs.espressif.com/projects/esp-idf/en/stable/esp32/api-reference/system/app_image_format.html#application-description>
esp_bootloader_esp_idf::esp_app_desc!();

#[allow(
    clippy::large_stack_frames,
    reason = "it's not unusual to allocate larger buffers etc. in main"
)]
#[main]
//main never returns 
fn main() -> ! {
    // generator version: 1.4.0
    // generator parameters: -o esp32

    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);
    let delay = Delay::new();

    //mut = mutable can change later, rust vars are constant by default
    let mut count: u32 = 0;
    //let v = [1, 2, 3];
    let mut led = Output::new(peripherals.GPIO13, Level::Low, OutputConfig::default());
    let button = Input::new(peripherals.GPIO0, InputConfig::default().with_pull(Pull::Up));
    let mut was_pressed = false;
    let mut press_start = Instant::now(); 
    //let delays = [200, 200, 200, 600, 600, 600, 200, 200, 200];
    loop {
        //*println!("{}", v[count as usize])
        //count in hex: {:x}
        /*
        for d in delays {
            led.set_high();
            delay.delay_millis(d);
            led.set_low();
            delay.delay_millis(200);
        } */
         
        let pressed = button.is_low();
        if pressed && !was_pressed {
            count += 1;
            println!("Pressed! {:x}", count);

            press_start = Instant::now();

            //debounce
            delay.delay_millis(20);
            if button.is_low() {
                led.toggle();
                println!("LED is now {}", if led.is_set_high() {"ON"} else {"LOW"});
            }
        }
        else if !pressed && was_pressed {
            let held_ms = press_start.elapsed().as_millis();
            println!("Released! Held for {} ms", held_ms);
        }
        was_pressed = pressed;
        delay.delay_millis(10);
    }

    // for inspiration have a look at the examples at https://github.com/esp-rs/esp-hal/tree/esp-hal-v1.2.2/examples
}
