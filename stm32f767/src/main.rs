/*
STM32F767 Rust project/template
Target hardware: STM32F767ZI (NUCLEO-F767ZI)

Author: Jonah Swain
*/

/* RUST EMBEDDED CONFIGURATION */
#![no_std] // No standard library
#![no_main] // No main (for OS)

/* DEPENDENCIES */
extern crate stm32f7; // STM32F7 standard peripheral library
extern crate panic_halt; // Panic handler halt program
extern crate cortex_m; // Cortex M low-level registers
extern crate cortex_m_rt; // Cortex M runtime

use cortex_m_rt::entry; // Cortex M runtime entry point
use stm32f7::stm32f7x7; // STM32F767 peripherals
use cortex_m::interrupt::{self, Mutex}; // Cortex M interrupt library, Mutex library
use cortex_m::asm::nop; // Cortex M No-operation function
use core::cell::RefCell; // Core library RefCell

/* GLOBALLY SHARED PERIPHERAL MUTEXES */
static _RCC: Mutex<RefCell<Option<stm32f7x7::RCC>>> = Mutex::new(RefCell::new(None)); // RCC
static _PWR: Mutex<RefCell<Option<stm32f7x7::PWR>>> = Mutex::new(RefCell::new(None)); // PWR
static _FLASH: Mutex<RefCell<Option<stm32f7x7::FLASH>>> = Mutex::new(RefCell::new(None)); // FLASH

/* MAIN FUNCTION */
#[entry] // Entry point for the Cortex M runtime
fn main() -> ! {
    /* INITIAL CONFIGURATION/SETUP */

    let stm32f7_peripherals = stm32f7x7::Peripherals::take().unwrap(); // Get STM32F767 peripherals
    
    interrupt::free(|cs| { // Store all global peripherals in static mutexes
        let __rcc = stm32f7_peripherals.RCC;
        _RCC.borrow(cs).replace(Some(__rcc));

        let __pwr = stm32f7_peripherals.PWR;
        _PWR.borrow(cs).replace(Some(__pwr));
        
        let __flash = stm32f7_peripherals.FLASH;
        _FLASH.borrow(cs).replace(Some(__flash));
    });

    conf_clk(); // Configure the mcu clock

    /* MAIN LOOP (INFINITE) */
    loop {

    }
}

/* ADDITIONAL FUNCTIONS */
fn conf_clk() { // Configure the mcu clock to 216MHz (HSE via PLL)
    /*
    The relevant clock rates are configured as follows:
    - System clock = PLL = 216MHz
    - AHB clock = PLL = 216MHz
    - Ethernet PTP clock = PLL = 216MHz
    - AHB HCLK (core/bus/DMA/Cortex system timer/Cortex FCLK) = AHB = PLL = 216MHz
    - APB1 peripheral clock = PLL/4 = 54MHz
    - APB1 timer clock = PLL/2 = 108MHz
    - APB2 peripheral clock = PLL/2 = 108MHz
    - APB2 timer clock = PLL = 216MHz

    The following other neccessary configuration changes are made:
    - Voltage scaling set to scale 1 (maximum performance)
    - Over-drive mode enabled
    - Flash latency set to 7 wait states (8 CPU cycles)
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

        pwr.cr1.modify(|_,w| w.vos().scale1()); // Set voltage scaling output to scale 1 (max performance)
        nop();

        rcc.cr.modify(|_,w| w.hseon().set_bit()); // Enable the HSE (crystal oscillator)
        while rcc.cr.read().hserdy().is_not_ready() {} // Wait for HSE to be stable (ready)

        rcc.pllcfgr.modify(|_,w| w.pllsrc().set_bit()); // Set PLL source to HSE
        nop();

        rcc.pllcfgr.modify(|_,w| unsafe{ // Configure PLL multipliers and dividers
            w.pllm().bits(8); // Input (HSE) division factor
            w.plln().bits(216); // VCO multiplication factor
            w.pllp().div2(); // System clock division factor
            w.pllq().bits(9) // USB/SDIO 48MHz clock division factor
        });
        nop();

        rcc.cr.modify(|_,w| w.pllon().set_bit()); // Enable the main PLL

        pwr.cr1.modify(|_,w| w.oden().set_bit()); // Enable over-drive mode
        while pwr.csr1.read().odrdy().bit_is_clear() {} // Wait for over-drive to be ready
        pwr.cr1.modify(|_,w| w.odswen().set_bit()); // Switch voltage regulator to over-drive mode
        while pwr.csr1.read().odswrdy().bit_is_clear() {} // Wait for voltage regulator to switch to over-drive mode

        while rcc.cr.read().pllrdy().is_not_ready() {} // Wait for PLL to be ready

        let brw_flash = _FLASH.borrow(cs).borrow(); // Borrow FLASH
        let flash = brw_flash.as_ref().unwrap(); // Unwrap FLASH

        flash.acr.modify(|_,w| w.latency().ws7()); // Set flash access latency to 3 wait states

        rcc.cfgr.modify(|_,w| { // Configure bus clock prescalers
            w.hpre().div1(); // AHB prescaler
            w.ppre1().div4(); // APB1 prescaler
            w.ppre2().div2() // APB2 prescaler
        });
        nop();

        rcc.cfgr.modify(|_,w| w.sw().pll()); // Set the system clock source to PLL
        while !rcc.cfgr.read().sws().is_pll() {} // Wait until the PLL is the selected system clock source

        rcc.dckcfgr2.modify(|_,w| w.ck48msel().clear_bit()); // Set the CLK48 source to PLL
    });
}
