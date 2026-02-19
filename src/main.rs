//! Calendar CLI application.
//!
//! # Usage
//! ```ignore
//! cal          // Current month
//! cal 2026     // Year 2026
//! cal 2 2026   // February 2026
//! cal -3       // Three months
//! cal -y       // Whole year
//! ```

use cal::args::{Args, get_display_date};
use cal::formatter::{
    print_month, print_months_count, print_three_months, print_twelve_months, print_year,
};
use cal::types::CalContext;

fn main() {
    let args = Args::parse();

    if let Err(e) = run(&args) {
        eprintln!("cal: {}", e);
        std::process::exit(1);
    }
}

fn run(args: &Args) -> Result<(), String> {
    let ctx = CalContext::new(args)?;
    let (year, month, _day) = get_display_date(args)?;

    // Display mode priority: year > twelve_months > three_months > months_count > single
    if args.year {
        print_year(&ctx, year);
    } else if args.twelve_months {
        print_twelve_months(&ctx, year, month);
    } else if args.three_months {
        print_three_months(&ctx, year, month);
    } else if let Some(count) = args.months_count {
        print_months_count(&ctx, year, month, count)?;
    } else {
        print_month(&ctx, year, month);
    }

    Ok(())
}
