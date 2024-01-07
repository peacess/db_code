pub(crate) fn to_snake_name(name: &String) -> String {
    let chs = name.chars();
    let mut new_name = String::new();
    let chs_len = name.len();
    for (index, x) in chs.enumerate() {
        if x.is_uppercase() {
            if index != 0 && (index + 1) != chs_len {
                new_name.push('_');
            }
            new_name.push_str(x.to_lowercase().to_string().as_str());
        } else {
            new_name.push(x);
        }
    }
    new_name
}
