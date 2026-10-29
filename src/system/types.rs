use crate::desktop::Installed;
use crate::mimeapps::Layer;
use crate::mimedb::MimeDb;
use crate::utils::Xdg;

#[derive(Debug, Clone)]
pub struct System {
    pub xdg: Xdg,
    pub installed: Installed,
    pub db: MimeDb,
    pub layers: Vec<Layer>,
}
