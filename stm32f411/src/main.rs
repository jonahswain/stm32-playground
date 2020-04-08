/*
STM32F411 Rust project/template
Target hardware: STM32F411CEU6 BlackPill

Author: Jonah Swain
*/

/* RUST EMBEDDED CONFIGURATION */
#![no_std] // No standard library
#![no_main] // No main (for OS)

/* DEPENDENCIES */
extern crate stm32f4; // STM32F4 standard peripheral library
extern crate panic_halt; // Panic handler halt program
extern crate cortex_m; // Cortex M low-level registers
extern crate cortex_m_rt; // Cortex M runtime

use cortex_m_rt::entry; // Cortex M runtime entry point
use stm32f4::stm32f411; // STM32F411 peripherals
use cortex_m::interrupt::{self, Mutex}; // Cortex M interrupt library, Mutex library
use cortex_m::asm::nop; // Cortex M No-operation function
use core::cell::RefCell; // Core library RefCell

/* GLOBALLY SHARED PERIPHERAL MUTEXES */
static _RCC: Mutex<RefCell<Option<stm32f411::RCC>>> = Mutex::new(RefCell::new(None)); // RCC
static _PWR: Mutex<RefCell<Option<stm32f411::PWR>>> = Mutex::new(RefCell::new(None)); // PWR
static _FLASH: Mutex<RefCell<Option<stm32f411::FLASH>>> = Mutex::new(RefCell::new(None)); // FLASH

/* MAIN FUNCTION */
#[entry] // Entry point for the Cortex M runtime
fn main() -> ! {
    /* INITIAL CONFIGURATION/SETUP */

    let stm32f4_peripherals = stm32f411::Peripherals::take().unwrap(); // Get STM32F411 peripherals
    
    interrupt::free(|cs| { // Store all global peripherals in static mutexes
        let __rcc = stm32f4_peripherals.RCC;
        _RCC.borrow(cs).replace(Some(__rcc));

        let __pwr = stm32f4_peripherals.PWR;
        _PWR.borrow(cs).replace(Some(__pwr));
        
        let __flash = stm32f4_peripherals.FLASH;
        _FLASH.borrow(cs).replace(Some(__flash));
    });

    conf_clk(); // Configure the mcu clock

    /* MAIN LOOP (INFINITE) */
    loop {

    }
}

/* ADDITIONAL FUNCTIONS */
fn conf_clk() { // Configure the mcu clock to 96MHz (HSE via PLL)
    /*
    The relevant clock rates are configured as follows:
    - System clock = PLL = 96MHz
    - AHB clock = PLL = 96MHz
    - Ethernet PTP clock = PLL = 96MHz
    - AHB HCLK (core/bus/DMA/Cortex system timer/Cortex FCLK) = AHB = PLL = 96MHz
    - APB1 peripheral clock = PLL/2 = 48MHz
    - APB1 timer clock = PLL = 96MHz
    - APB2 = PLL = 96MHz

    The following other neccessary configuration changes are made:
    - Voltage scaling set to scale 1 (maximum performance)
    - Flash latency set to 3 wait states (4 CPU cycles)
    */

    interrupt::free(|cs| { // Critical section (for safe use of shared/global peripherals)

        let brw_rcc = _RCC.borrow(cs).borrow(); // Borrow RCC
        let rcc = brw_rcc.as_ref().unwrap(); // Unwrap RCC

        rcc.cr.modify(|_,w| w.pllon().clear_bit()); // Disable the main PLL
        while rcc.cr.read().pllon().is_on() {} // Wait until main PLL is disabled

        rcc.apb1enr.modify(|_,w| w.pwren().set_bit()); // Enable clock for PWR controller
        nop();

        let brw_pwr = _PWR.borrow(cs).borrow(); // Borrow PWR
        let pwr = brw_pwr.as_ref().unwrap(); // Unwrap PWR

        pwr.cr.modify(|_,w| unsafe{w.vos().bits(3_u8)}); // Set voltage scaling output to scale 1 (max performance)
        nop();

        rcc.cr.modify(|_,w| w.hseon().set_bit()); // Enable the HSE (crystal oscillator)
        while rcc.cr.read().hserdy().is_not_ready() {} // Wait for HSE to be stable (ready)

        rcc.pllcfgr.modify(|_,w| w.pllsrc().set_bit()); // Set PLL source to HSE
        nop();

        rcc.pllcfgr.modify(|_,w| unsafe{ // Configure PLL multipliers and dividers
            w.pllm().bits(25_u8); // Input (HSE) division factor
            w.plln().bits(192_u16); // VCO multiplication factor
            w.pllp().div2(); // System clock division factor
            w.pllq().bits(4_u8) // USB/SDIO 48MHz clock division factor
        });
        nop();

        rcc.cr.modify(|_,w| w.pllon().set_bit()); // Enable the main PLL
        while rcc.cr.read().pllrdy().is_not_ready() {} // Wait for PLL to be ready

        let brw_flash = _FLASH.borrow(cs).borrow(); // Borrow FLASH
        let flash = brw_flash.as_ref().unwrap(); // Unwrap FLASH

        flash.acr.modify(|_,w| unsafe{w.latency().bits(3_u8)}); // Set flash access latency to 3 wait states

        rcc.cfgr.modify(|_,w| { // Configure bus clock prescalers
            w.hpre().div1(); // AHB prescaler
            w.ppre1().div2(); // APB1 prescaler
            w.ppre2().div1() // APB2 prescaler
        });
        nop();

        rcc.cfgr.modify(|_,w| w.sw().pll()); // Set the system clock source to PLL
        while !rcc.cfgr.read().sws().is_pll() {} // Wait until the PLL is the selected system clock source
    });
}