use strum_macros::EnumIter;
#[derive(Copy, Clone, Eq, PartialEq, Debug, EnumIter)]
pub enum Element {
    None,
    Sand,
}
