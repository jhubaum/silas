use super::Mode;
use super::website_new;

pub struct LayoutInfo {

}

impl LayoutInfo {
    pub fn new(website: &website_new::Website) -> Self {
        LayoutInfo {  }
    }
}

impl website_new::Website {
    pub fn serialize<T: Mode>(&self, mode: &T, layout: &LayoutInfo) -> String {
        String::from("")
    }
}

impl website_new::Project {
    pub fn serialize<T: Mode>(&self, mode: &T, layout: &LayoutInfo) -> String {
        String::from("")
    }
}

impl website_new::OrgFile {
    pub fn serialize<T: Mode>(&self, mode: &T, layout: &LayoutInfo) -> String {
        String::from("")
    }
}
