// This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0. If a copy of
// the MPL was not distributed with this file, You can obtain one at <http://mozilla.org/MPL/2.0/>.

use argh::FromArgs;

/// Flow - A modern workflow engine
#[derive(FromArgs)]
struct Args {
    /// print version information
    #[argh(switch)]
    version: bool,
}

fn main() {
    let args: Args = argh::from_env();

    if args.version {
        println!("flow {}", env!("CARGO_PKG_VERSION"));
        return;
    }

    println!("Hello, world!");
}

// End of File
