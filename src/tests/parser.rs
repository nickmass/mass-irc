use irc::CommandParser;

const EXAMPLES: &'static str = include_str!("parser_examples.txt");

#[test]
fn parser_full() {
    let test_cases = EXAMPLES.lines().map(|x| {
        let mut test = String::from(x);
        test.push_str("\r\n");
        test
    });

    let parser = CommandParser::new();

    for test in test_cases {
        parser.parse(&test.as_bytes().to_vec());
    }
}

