#![no_std]
#![no_main]
#![deny(
    clippy::mem_forget,
    reason = "mem::forget is generally not safe to do with esp_hal types, especially those \
    holding buffers for the duration of a data transfer."
)]
#![deny(clippy::large_stack_frames)]

use embassy_executor::Spawner;
use embassy_time::{Duration, Timer};
use esp_hal::clock::CpuClock;
use esp_hal::gpio::{Level, Output, OutputConfig, Input, InputConfig, Pull};
use esp_hal::timer::timg::TimerGroup;
use esp_println::println;
use esp_hal::time::Instant;  
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::signal::Signal;

static PRESSED: Signal<CriticalSectionRawMutex, ()> = Signal::new();

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    println!("PANIC: {}", info);
    loop {}
}

// This creates a default app-descriptor required by the esp-idf bootloader.
// For more information see: <https://docs.espressif.com/projects/esp-idf/en/stable/esp32/api-reference/system/app_image_format.html#application-description>
esp_bootloader_esp_idf::esp_app_desc!();


#[embassy_executor::task] 
async fn sos(mut led: Output<'static>) {
    loop {
        PRESSED.wait().await;
        let delays = [200, 200, 200, 600, 600, 600, 200, 200, 200];
        for d in delays {
            led.set_high();
            Timer::after(Duration::from_millis(d)).await;
            led.set_low();
            Timer::after(Duration::from_millis(200)).await;
        } 
        PRESSED.reset(); // forget any presses that came in while blinking
    }

}

//spawn always requires static (valid for whole program)
#[embassy_executor::task] //turns async fn into a task scheduler can run
async fn blink(mut led: Output<'static>) {
    loop {
        led.toggle();
        Timer::after(Duration::from_millis(250)).await;
    }
}

#[embassy_executor::task]
async fn watch_button(mut button: Input<'static>) {
    let mut count: u32 = 0;
    loop {
        //interrupt, CPU idle
        button.wait_for_low().await;
        let press_start = Instant::now();

        Timer::after(Duration::from_millis(20)).await; //debounce
        if button.is_high() {
            continue; //just a bounce
        }

        count += 1;
        println!("Pressed! Hex: {:x}", count);
        PRESSED.signal(()); //() so carries no data, just that PRESSED has been sent

        button.wait_for_high().await;
        let held_ms = press_start.elapsed().as_millis();
        println!("Released! Held for {} ms", held_ms);

        //let release bounce settle too
        Timer::after(Duration::from_millis(10)).await;
    }

}


#[allow(
    clippy::large_stack_frames,
    reason = "it's not unusual to allocate larger buffers etc. in main"
)]

//main never returns 
#[esp_rtos::main]
async fn main(spawner: Spawner) -> ! {
    // generator version: 1.4.0
    // generator parameters: -o esp32

    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);

    let timg0 = TimerGroup::new(peripherals.TIMG0);
    esp_rtos::start(timg0.timer0, peripherals.FROM_CPU_INTR0);

    //mut = mutable can change later, rust vars are constant by default
    let led13 = Output::new(peripherals.GPIO13, Level::Low, OutputConfig::default());
    let led2 = Output::new(peripherals.GPIO2, Level::Low, OutputConfig::default());
    let button = Input::new(peripherals.GPIO0, InputConfig::default().with_pull(Pull::Up));

    spawner.spawn(blink(led2).unwrap());
    spawner.spawn(sos(led13).unwrap());
    spawner.spawn(watch_button(button).unwrap());

    loop {         
        Timer::after(Duration::from_millis(10)).await; //let other tasks run
    }

    // for inspiration have a look at the examples at https://github.com/esp-rs/esp-hal/tree/esp-hal-v1.2.2/examples
}
