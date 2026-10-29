use super::types::MimeApps;

pub fn render(file: &MimeApps) -> String {
    file.lines.iter().map(|line| line.raw.as_str()).collect()
}
