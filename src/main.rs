// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: Emily Albini <emily@emilyalbini.it>

mod replace;

use crate::replace::replace_username;
use anyhow::Error;
use std::path::Path;

fn main() -> Result<(), Error> {
    let mut args = std::env::args().skip(1);
    let Some(input) = args.next() else { usage() };
    let Some(output) = args.next() else { usage() };
    let Some(old_username) = args.next() else {
        usage()
    };
    let Some(new_username) = args.next() else {
        usage()
    };
    if args.next().is_some() {
        usage();
    }

    replace_username(Path::new(&input), Path::new(&output), &old_username, &new_username)
}

fn usage() -> ! {
    eprintln!(
        "usage: {} <save-file> <output-file> <old-username> <new-username>",
        std::env::args().next().unwrap()
    );
    std::process::exit(1);
}
