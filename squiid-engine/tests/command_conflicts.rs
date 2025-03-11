use squiid_engine::command_mappings;

// test that no commands contain each other
// this creates an impossible command in RPN mode
// ex: log and blog
// gt and egt

fn check_commands(strings: &Vec<String>) -> Option<(String, String)> {
    for (i, s1) in strings.iter().enumerate() {
        for (j, s2) in strings.iter().enumerate() {
            if i != j && s1.len() < s2.len() && s2.starts_with(s1) {
                return Some((s1.clone(), s2.clone()));
            }
        }
    }
    None
}

#[test]
fn test_no_conflicts() {
    let commands = command_mappings::create_function_map()
        .keys()
        .map(|k| k.to_owned())
        .collect();
    assert_eq!(check_commands(&commands), None);
}
