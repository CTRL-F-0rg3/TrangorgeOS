use crate::vga_buffer::Color;
use crate::{print, print_colored, println};

pub type TestResult = Result<&'static str, &'static str>;

pub struct Test {
    pub module: &'static str,
    pub func: fn() -> TestResult,
}

#[macro_export]
macro_rules! test_module {
    ($body:block) => {
        pub fn self_test() -> $crate::testing::TestResult {
            $body
        }
    };
}

pub fn run_test(test: &Test) -> bool {
    print_colored!(Color::Magenta, "{}", test.module);
    print!(" ");
    match (test.func)() {
        Ok(msg) => {
            print!("{} ", msg);
            print!("[");
            print_colored!(Color::LightGreen, "OK");
            println!("]");
            true
        }
        Err(msg) => {
            print!("[");
            print_colored!(Color::LightRed, "FAILED");
            print!("]");
            println!(" {}", msg);
            false
        }
    }
}

pub fn run_all(tests: &[Test]) {
    println!("Running {} module test(s)...", tests.len());
    let mut passed = 0;
    for test in tests {
        if run_test(test) {
            passed += 1;
        }
    }
    println!();
    if passed == tests.len() {
        print_colored!(Color::LightGreen, "SYSTEM STATUS: {}/{} OK", passed, tests.len());
    } else {
        print_colored!(
            Color::LightRed,
            "SYSTEM STATUS: {}/{} FAILED",
            tests.len() - passed,
            tests.len()
        );
    }
    println!();
}
