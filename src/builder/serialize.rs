use super::Mode;
use super::website_new;
use super::website_new::BlogElement;
use serde::Serialize;

#[derive(Serialize)]
pub struct LayoutInfo {
    header: Vec<SerializedLink>,
    #[serde(rename = "website-name")]
    website_name: SerializedLink
}

#[derive(Serialize)]
pub struct SerializedLink {
    target: String,
    title: String
}

#[derive(Serialize)]
pub struct SerializedPost<'a> {
    layout: &'a LayoutInfo
}

#[derive(Serialize)]
pub struct SerializedProjectIndex<'a> {
    layout: &'a LayoutInfo
}

impl SerializedLink {
    fn from_blog_element<TElem: BlogElement, TMode: Mode>(
        elem: &TElem,
        website: &website_new::Website,
        mode: &TMode) -> Self {

        SerializedLink {
            target: elem.url(website, mode.base_url()),
            title: elem.title().to_string()
        }
    }
}


impl LayoutInfo {
    pub fn new<T: Mode>(website: &website_new::Website, mode: &T) -> Self {
        let mut header = Vec::new();
        for page in website.pages.values() {
            if mode.include_page(page) {
                let link = SerializedLink::from_blog_element(page, website, mode);
                header.push(link);
            }
        }

        for proj in website.projects.values() {
            let link = SerializedLink::from_blog_element(proj, website, mode);
            header.push(link);
        }

        LayoutInfo {
            header,
            website_name: SerializedLink::from_blog_element(website, website, mode)
        }
    }
}

impl website_new::Website {
    pub fn serialize<'a, T: Mode>(&self, mode: &T, layout: &'a LayoutInfo) -> SerializedPost<'a> {
        self.page_by_id("about").unwrap().serialize(mode, layout)
    }
}

impl website_new::Project {
    pub fn serialize<'a, T: Mode>(&self, mode: &T, layout: &'a LayoutInfo) -> SerializedProjectIndex<'a> {
        SerializedProjectIndex { layout }
    }
}

impl website_new::OrgFile {
    pub fn serialize<'a, T: Mode>(&self, mode: &T, layout: &'a LayoutInfo) -> SerializedPost<'a> {
        SerializedPost { layout }
    }
}
