#[derive(Clone, Debug)]
pub struct Filter {
    pub id: u32,
    pub name: String,
    pub field: String,
    pub pattern: String,
}
