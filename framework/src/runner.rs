pub use crate::{
    inputs::Inputs,
    outputs::ColoredOutput,
    result::{IntoResult, Result},
};
pub use colored::Colorize;
pub use std::io::Write;

#[macro_export]
macro_rules! main {
    ($($day:ident),*$(,)?) => {
        $(mod $day;)*

        fn main() -> $crate::runner::Result<()> {
            use $crate::runner::*;
            println!(
                "\n🎄 {} {} {} {} 🎄\n",
                "Advent".bright_red().bold(),
                "of".bright_green(),
                "Code".blue().bold(),
                "2023".bright_magenta().bold()
            );

            let included_days: Vec<u32> = std::env::args()
                .filter_map(|v| v.parse::<u32>().ok())
                .collect();

            let mut duration = std::time::Duration::ZERO;
            let mut inputs = Inputs::new();
            $({
                if included_days.is_empty() || included_days.contains(&$day::DayMetadata::number()) {
                    $day::DayMetadata::execute(&mut inputs, &mut duration)?;
                }
            })*
            let after = std::time::Instant::now();
            println!();
            println!("{:?}", duration);
            Ok(())
        }
    };
}

#[macro_export]
macro_rules! day {
    ($day_nr:literal, $parse_fn:ident => $($part_fn:ident),+) => {
use super::prelude::*;
pub struct DayMetadata;
impl DayMetadata {
    pub fn number() -> u32 { $day_nr }
    pub fn execute(inputs: &mut $crate::runner::Inputs, duration: &mut std::time::Duration) -> $crate::runner::Result<()> {
        use $crate::runner::*;
        const OUTPUT_WIDTH: usize = 40;
        print!(
            "{} {}",
            "Day".bright_blue(),
            format!("{:>2}", $day_nr).bright_red().bold()
        );

        let input = inputs.get($day_nr)?;
        let before = std::time::Instant::now();
        let parsed = $parse_fn(&input)?;
        *duration += before.elapsed();
        $({
            let part_name = stringify!($part_fn);
            let remaining_space = OUTPUT_WIDTH.checked_sub(part_name.len() + 1).unwrap_or(0);
            print!(" {} {} ", "::".magenta(), part_name.bright_yellow());
            _ = std::io::stdout().flush();
            let before = std::time::Instant::now();
            let result = $part_fn(&parsed);
            *duration += before.elapsed();
            let result: ColoredOutput = IntoResult::into_result(result)?.into();
            let str_len = result.value().len() - result.control_count();
            let remaining_space = remaining_space.checked_sub(str_len).unwrap_or(0);
            for _ in 0..remaining_space {
                print!(" ");
            }
            print!("{}", result.value());
            _ = std::io::stdout().flush();
        })+
        println!();

        Ok(())
    }
}
$crate::paste! {
    #[cfg(feature = "criterion")]
    #[criterion_macro::criterion]
    pub fn benchmarks(c: &mut criterion::Criterion) {
        use criterion::{black_box, Criterion};
        let mut inputs = $crate::inputs::Inputs::new();
        let input = inputs.get($day_nr).expect("could not get input");
        let parsed = $parse_fn(&input).expect("could not parse input");
        c.bench_function(stringify!([<day $day_nr _ $parse_fn>]), |b| b.iter(|| $parse_fn(&input)));
        $(
            c.bench_function(stringify!([<day $day_nr _ $part_fn>]), |b| b.iter(|| $part_fn(&parsed)));
        )*
    }
}
    };
}

#[macro_export]
macro_rules! tests {
    ($($x:tt)*) => {
        #[cfg(test)]
        #[cfg(not(feature = "criterion"))]
        mod tests {
            use super::*;
            use $crate::test_pt;

            $($x)*
        }
    };
}

#[macro_export]
macro_rules! test_pt {
    ($parse_fn:ident, $test_name:ident, |$input_name:ident| $part_logic:block, $($input:expr => $output:expr),+$(,)?) => {
#[test]
fn $test_name() {
    use $crate::runner::*;
    $(
        let $input_name = match IntoResult::into_result(super::$parse_fn($input)) {
            Ok(x) => x,
            Err(e) => panic!("parsing failed: {e}\ninput: {:?}", String::from_utf8_lossy($input).into_owned()),
        };
        let result = match IntoResult::into_result($part_logic) {
            Ok(x) => x,
            Err(e) => panic!("execution failed: {e}\ninput: {:?}", String::from_utf8_lossy($input).into_owned()),
        };
        let output = $output;
        if result != output {
            panic!("incorrect output, expected: {:?}, got: {:?}\ninput: {:?}", output, result, String::from_utf8_lossy($input).into_owned());
        }
    )+
}
    };
    ($parse_fn:ident, $pt_fn:ident, $($input:expr => $output:expr),+$(,)?) => {
        $crate::test_pt!($parse_fn, $pt_fn, |input| { super::$pt_fn(&input) }, $($input => $output),+);
    };
}
