#![allow(clippy::empty_loop)]
#![no_std]
#![no_main]
#![feature(isa_attribute)]

#[macro_use]
extern crate gba;
use gba::prelude::*;

#[panic_handler]
#[allow(unused)]
fn panic(info: &core::panic::PanicInfo) -> ! {
	fatal!("{}", info);

	loop {
		DISPCNT.read();
	}
}

#[no_mangle]
pub fn main() -> ! {
	// Use graphics mode 3
	const SETTING: DisplayControl = DisplayControl::new()
		.with_display_mode(3)
		.with_display_bg2(true);
	DISPCNT.write(SETTING);

	// Set the IRQ handler to use.
	unsafe { USER_IRQ_HANDLER.write(Some(irq_handler_a32)) };

	// Enable all interrupts that are set in the IE register.
	unsafe { IME.write(true) };

	// Request that VBlank, HBlank and VCount will generate IRQs.
	const DISPLAY_SETTINGS: DisplayStatus = DisplayStatus::new()
		.with_vblank_irq_enabled(true)
		.with_hblank_irq_enabled(true)
		.with_vcount_irq_enabled(true);
	DISPSTAT.write(DISPLAY_SETTINGS);

	let colors = [
		Color::from_rgb(255, 0, 0),
		Color::from_rgb(0, 255, 0),
		Color::from_rgb(0, 0, 255),
	];
	let mut index = 0usize;
	let mut lockout = false;
	mode3::dma3_clear_to(colors[index]);
	loop {
		let keys: Keys = KEYINPUT.read().into();
		if keys.a() {
			if !lockout {
				index += 1;
				index %= colors.len();
				lockout = true;
				mode3::dma3_clear_to(colors[index]);
			}
		} else {
			lockout = false;
		}

		// Request the vblank interrupt
		let flags = InterruptFlags::new().with_vblank(true);
		unsafe { IE.write(flags) };

		// Wait for nex vertical blank
		unsafe { VBlankIntrWait() };
	}
}

#[instruction_set(arm::a32)]
extern "C" fn irq_handler_a32() {
	// we just use this a32 function to jump over back to t32 code.
	irq_handler_t32()
}

fn irq_handler_t32() {
	// disable Interrupt Master Enable to prevent an interrupt during the handler
	unsafe { IME.write(false) };

	// read which interrupts are pending, and "filter" the selection by which are
	// supposed to be enabled.
	let which_interrupts_to_handle = IRQ_PENDING.read() & IE.read();

	// read the current IntrWait value. It sorta works like a running total, so
	// any interrupts we process we'll enable in this value, which we write back
	// at the end.
	let mut intr_wait_flags = INTR_WAIT_ACKNOWLEDGE.read();

	if which_interrupts_to_handle.vblank() {
		vblank_handler();
		intr_wait_flags.set_vblank(true);
	}
	if which_interrupts_to_handle.hblank() {
		hblank_handler();
		intr_wait_flags.set_hblank(true);
	}
	if which_interrupts_to_handle.vcount() {
		vcount_handler();
		intr_wait_flags.set_vcount(true);
	}
	if which_interrupts_to_handle.timer0() {
		timer0_handler();
		intr_wait_flags.set_timer0(true);
	}
	if which_interrupts_to_handle.timer1() {
		timer1_handler();
		intr_wait_flags.set_timer1(true);
	}

	// acknowledge that we did stuff.
	IRQ_ACKNOWLEDGE.write(which_interrupts_to_handle);

	// write out any IntrWait changes.
	unsafe { INTR_WAIT_ACKNOWLEDGE.write(intr_wait_flags) };

	// re-enable as we go out.
	unsafe { IME.write(true) };
}

fn vblank_handler() {}

fn hblank_handler() {}

fn vcount_handler() {}

fn timer0_handler() {}

fn timer1_handler() {}
