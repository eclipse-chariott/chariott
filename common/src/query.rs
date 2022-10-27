// Copyright (c) Microsoft Corporation. All rights reserved.
// Licensed under the MIT license.

use regex::Regex;

pub fn regex_pattern_from_query(query: &str) -> String {
    format!("^{}$", query.replace("**", ".{0,}").replace('*', "[^.]{0,}"))
}

pub fn regex_from_query(query: &str) -> Regex {
    Regex::new(&regex_pattern_from_query(query)).unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn test_regex_query_conversion() {
        test(false, "foo", "foobar");
        test(true, "foo*", "foobar");
        test(true, "foo*", "foobarbaz");
        test(true, "*baz", "*baz");
        test(false, "*bar", "foobarbaz");
        test(false, "bar*", "foobarbaz");
        test(true, "*bar*", "foobarbaz");
        test(true, "vdt.cabin.*.temp*", "vdt.cabin.hvac.temperature");
        test(true, "vdt.**.temp*", "vdt.cabin.hvac.temperature");
        test(false, "temp*erature", "temp.erature");
        test(true, "**.doors.**.lock", "vdt.cabin.doors.door1.lock");

        fn test(expected: bool, query: &str, input: &str) {
            let regex = regex_from_query(query);
            assert_eq!(
                expected,
                regex.is_match(input),
                "{expected} = query: {query} -> {}; input: {input}",
                regex_pattern_from_query(query)
            );
        }
    }
}
