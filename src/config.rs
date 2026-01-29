pub struct Conf {
    pub usercolor: u8,
    pub errorcolor: u8,
    pub separator: String
}
impl Conf {
    pub fn make_conf(_noconf: bool) -> Self {
        Self {
            usercolor: 0,
            errorcolor: 0,
            separator: "->".to_string()
        }
    }
}