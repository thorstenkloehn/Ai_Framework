#[derive(Debug, Clone, PartialEq)]
pub enum Rolle {
    System,
    Benutzer,
    Assistent,
}
#[derive(Debug, Clone)]
pub struct Nachricht {
    pub rolle: Rolle,
    pub inhalt: String,
}
impl Nachricht {
    pub fn neu(rolle: Rolle, inhalt: impl Into<String>) -> Self {
        Nachricht {
            rolle,
            inhalt: inhalt.into(),
        }
    }
}
