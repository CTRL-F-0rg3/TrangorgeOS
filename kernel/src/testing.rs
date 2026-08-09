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

pub fn run_test(test: &Test) {
    print_colored!(Color::Magenta, "{}", test.module);
    print!(" ");
    match (test.func)() {
        Ok(msg) => {
            print!("{} ", msg);
            print!("[");
            print_colored!(Color::LightGreen, "OK");
            println!("]");
        }
        Err(msg) => {
            print!("[");
            print_colored!(Color::LightRed, "FAILED");
            print!("]");
            println!(" {}", msg);
        }
    }
}

pub fn run_all(tests: &[Test]) {
    println!("Running {} module test(s)...", tests.len());
    for test in tests {
        run_test(test);
    }
    println!();
}
