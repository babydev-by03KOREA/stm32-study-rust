/*#![no_std] // 표준 라이브러리 없음 (OS가 없으니까)
#![no_main] // 일반적인 main 진입 방식 대신 cortex-m-rt의 #[entry] 사용

use core::ptr::{read_volatile, write_volatile};
use cortex_m_rt::entry;
use defmt::info;
use {defmt_rtt as _, panic_probe as _};

// C와 같은 레지스터 주소 (RM0008 기준)
const RCC_APB2ENR: *mut u32 = 0x4002_1018 as *mut u32;
const GPIOA_CRL: *mut u32 = 0x4001_0800 as *mut u32; // PA0~PA7 모드 설정
const GPIOA_IDR: *const u32 = 0x4001_0808 as *const u32; // PA 입력 값 (읽기 전용)
const GPIOA_ODR: *mut u32 = 0x4001_080C as *mut u32; // 입력 모드에선 풀업/풀다운 선택용
const GPIOC_CRH: *mut u32 = 0x4001_1004 as *mut u32;
const GPIOC_BSRR: *mut u32 = 0x4001_1010 as *mut u32; // 핀 켜기/끄기 전용

#[entry]
fn main() -> ! {
    info!("button demo start");

    unsafe {
        // 1. GPIOA(2번 비트), GPIOC(4번 비트)에 클럭 공급
        let apb2 = read_volatile(RCC_APB2ENR);
        write_volatile(RCC_APB2ENR, apb2 | (1 << 2) | (1 << 4));

        // 2. PA0 = 입력 + 풀업/풀다운 모드 (CNF=10, MODE=00 → 0b1000)
        let mut crl = read_volatile(GPIOA_CRL);
        crl &= !(0xF << 0);
        crl |= 0x8 << 0;
        write_volatile(GPIOA_CRL, crl);

        //    ODR의 0번 비트를 1로 → 풀업 선택 (0이면 풀다운)
        let odr = read_volatile(GPIOA_ODR);
        write_volatile(GPIOA_ODR, odr | (1 << 0));

        // 3. PC13 = 출력 (오늘 한 것과 같음)
        let mut crh = read_volatile(GPIOC_CRH);
        crh &= !(0xF << 20);
        crh |= 0x2 << 20;
        write_volatile(GPIOC_CRH, crh);
    }

    let mut was_pressed = false;

    loop {
        // 풀업이라 평소엔 1, 누르면 GND와 연결돼서 0
        let pressed = unsafe { (read_volatile(GPIOA_IDR) & (1 << 0)) == 0 };

        unsafe {
            if pressed {
                write_volatile(GPIOC_BSRR, 1 << (13 + 16)); // PC13 = 0 → LED 켜짐
            } else {
                write_volatile(GPIOC_BSRR, 1 << 13); // PC13 = 1 → LED 꺼짐
            }
        }

        // 상태가 바뀔 때만 로그
        if pressed != was_pressed {
            info!("button {}", if pressed { "pressed" } else { "released" });
            was_pressed = pressed;
        }
    }
}
*/

#![no_std]
#![no_main]

use cortex_m_rt::entry;
use defmt::info;
use embassy_stm32::gpio::{Input, Level, Output, Pull, Speed};
use {defmt_rtt as _, panic_probe as _};

#[entry]
fn main() -> ! {
    // 클럭 등 칩 기본 초기화 (C의 HAL_Init + SystemClock_Config 역할)
    let p = embassy_stm32::init(Default::default());
    info!("button demo (HAL) start");

    // CubeMX에서 클릭으로 하던 핀 설정을 코드로
    let button = Input::new(p.PA0, Pull::Up);                    // PA0: 입력, 풀업
    let mut led = Output::new(p.PC13, Level::High, Speed::Low);  // PC13: 출력, 처음엔 High(꺼짐)

    let mut was_pressed = false;

    loop {
        let pressed = button.is_low();   // 풀업이라 누르면 Low

        if pressed {
            led.set_low();    // Active-Low라 Low가 켜짐
        } else {
            led.set_high();
        }

        if pressed != was_pressed {
            info!("button {}", if pressed { "pressed" } else { "released" });
            was_pressed = pressed;
        }
    }
}