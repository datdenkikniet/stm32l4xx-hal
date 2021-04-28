#![no_main]
#![no_std]

use panic_rtt_target as _;

#[rtic::app(device = stm32l4xx_hal::pac, dispatchers = [USART1, USART2])]
mod app {
    use dwt_systick_monotonic::DwtSystick;
    use rtic::time::duration::*;
    use rtt_target::{rprintln, rtt_init_print};
    use stm32l4xx_hal::{i2c::I2c, prelude::*, rcc::MsiFreq};

    #[monotonic(binds = SysTick, default = true)]
    type DwtMono = DwtSystick<16_000_000>;

    #[init]
    fn init(cx: init::Context) -> (init::LateResources, init::Monotonics) {
        let mut flash = cx.device.FLASH.constrain();
        let mut rcc = cx.device.RCC.constrain();
        let mut pwr = cx.device.PWR.constrain(&mut rcc.apb1r1);
        let mut dcb = cx.core.DCB;
        let mut gpiob = cx.device.GPIOB.split(&mut rcc.ahb2);
        let dwt = cx.core.DWT;
        let systick = cx.core.SYST;
        let i2c = cx.device.I2C1;

        rtt_init_print!(NoBlockSkip, 4096);

        rprintln!("pre init");

        //
        // Initialize the clocks
        //
        let clocks = rcc
            .cfgr
            .msi(MsiFreq::RANGE32M)
            .freeze(&mut flash.acr, &mut pwr);

        // Setup the monotonic timer
        let mono2 = DwtSystick::new(
            &mut dcb,
            dwt,
            systick,
            clocks.msi().unwrap().to_hertz().0 / 2,
        );

        let mut pin_b8 = gpiob
            .pb8
            .into_open_drain_output(&mut gpiob.moder, &mut gpiob.otyper);
        pin_b8.internal_pull_up(&mut gpiob.pupdr, true);
        let acc_scl = pin_b8.into_af4(&mut gpiob.moder, &mut gpiob.afrh);

        let mut pin_b9 = gpiob
            .pb9
            .into_open_drain_output(&mut gpiob.moder, &mut gpiob.otyper);
        pin_b9.internal_pull_up(&mut gpiob.pupdr, true);
        let acc_sda = pin_b9.into_af4(&mut gpiob.moder, &mut gpiob.afrh);

        // May actually not be 200 khz, but at least it works (seems to be closer to 300 khz)
        let mut i2c = I2c::i2c1(i2c, (acc_scl, acc_sda), 400.khz(), clocks, &mut rcc.apb1r1);

        match i2c.write(0b01010101, &[0u8, 1u8, 2u8]) {
            core::result::Result::Ok(_) => {}
            core::result::Result::Err(e) => {
                panic!("{:?}", e)
            }
        }

        rprintln!("init");

        printer::spawn(1).unwrap();
        printer::spawn_after(Milliseconds(5_000_u32), 6).unwrap();
        printer::spawn_after(Milliseconds(6_000_u32), 7).unwrap();
        printer::spawn_after(Milliseconds(7_000_u32), 8).unwrap();
        printer::spawn_after(Milliseconds(8_000_u32), 9).unwrap();
        printer::spawn_after(Milliseconds(4_000_u32), 5).unwrap();
        printer::spawn_after(Milliseconds(3_000_u32), 4).unwrap();
        printer::spawn_after(Milliseconds(2_000_u32), 3).unwrap();
        printer::spawn_after(Milliseconds(1_000_u32), 2).unwrap();

        // (init::LateResources {}, init::Monotonics(mono2))
        (init::LateResources {}, init::Monotonics(mono2))
    }

    use core::convert::TryInto;

    pub type TEST = u32;

    #[task(capacity = 16)]
    fn printer(_cx: printer::Context, val: TEST) {
        let now: Milliseconds = monotonics::DwtMono::now()
            .duration_since_epoch()
            .try_into()
            .unwrap();
        rprintln!("Val: {} at {} ms", val, now.integer());
    }

    #[idle]
    fn idle(_: idle::Context) -> ! {
        rprintln!("idle");

        loop {
            cortex_m::asm::nop();
        }
    }
}
