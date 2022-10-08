#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(test_runner)]
#![reexport_test_harness_main = "test_main"]

use core::panic::PanicInfo;
use blog_os::{QemuExitCode, exit_qemu, serial_println, serial_print};

#[no_mangle]
pub extern "C" fn _start() -> ! {
    should_fail();
    serial_println!("[test did not panic]");
    exit_qemu(QemuExitCode::Failed);
    loop {}
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    serial_println!("[ok]");
    exit_qemu(QemuExitCode::Success);
    loop {}
}

// 在Cargo.toml中设置后可以去掉test case和test runner

// pub fn test_runner(tests: &[&dyn Fn()]) {
//     serial_println!("Running {} tests", tests.len());
//     for test in tests {
//         test();
//         serial_println!("[test did not panic]");
//         exit_qemu(QemuExitCode::Failed);
//     }
//     exit_qemu(QemuExitCode::Success);
// }

// 定义了自己的runner而没有复用

// 因为这个测试我们是希望那些本应该panic的函数, 因此如果正常执行了反而是不希望的结果, 因此我们就返回failed code并输出提升信息

// 这个做法的缺点在于只能测试第一个test case函数 (因为测试正常的话该函数会按预期panic, 那么后面的函数就不会被执行了)

// #[test_case]
fn should_fail() {
    serial_print!("should_fail... ");
    assert_eq!(0, 1);
}