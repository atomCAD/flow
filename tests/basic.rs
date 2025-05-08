// This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0.
// If a copy of the MPL was not distributed with this file,
// You can obtain one at <https://mozilla.org/MPL/2.0/>.

#[test]
fn test_sanity() {
    // A basic sanity test to verify the crate can be used
    let result = rivulet::add(2, 2);
    assert_eq!(result, 4);
}

// EOF
