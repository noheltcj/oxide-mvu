use alloc::vec::Vec;

use embassy_nrf::{bind_interrupts, peripherals, uarte};
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::channel::Channel;

bind_interrupts!(pub struct UartIrqs {
    UARTE0 => uarte::InterruptHandler<peripherals::UARTE0>;
});

static UART_CHANNEL: Channel<CriticalSectionRawMutex, Vec<u8>, 16> = Channel::new();

#[embassy_executor::task]
pub async fn uart_task(mut tx: uarte::UarteTx<'static>) {
    loop {
        let message = UART_CHANNEL.receive().await;
        let _ = tx.write(&message).await;
    }
}

pub fn uart_enqueue_str(msg: &str) {
    let mut buf = Vec::with_capacity(msg.len() + 2);
    buf.extend_from_slice(msg.as_bytes());
    buf.extend_from_slice(b"\r\n");
    let _ = UART_CHANNEL.try_send(buf);
}

#[macro_export]
macro_rules! uart_println {
    () => {{
        $crate::uart::uart_enqueue_str("");
    }};
    ($($arg:tt)*) => {{
        let msg = alloc::format!($($arg)*);
        $crate::uart::uart_enqueue_str(&msg);
    }};
}
