use super::Mode;
use super::website_new;
use super::website_new::BlogElement;
use serde::Serialize;

#[derive(Serialize)]
pub struct LayoutInfo {
    header: Vec<SerializedLink>,
    #[serde(rename = "website-name")]
    website_name: SerializedLink,
    #[serde(rename = "base-url")]
    base_url: String
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
struct PostSummary<'a> {
    heading: &'a str,
    id: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    published: Option<chrono::naive::NaiveDate>,
    #[serde(skip_serializing_if = "Option::is_none")]
    summary: Option<&'a str>
}

#[derive(Serialize)]
pub struct SerializedProjectIndex<'a> {
    layout: &'a LayoutInfo,
    title: String,
    heading: String,
    posts: Vec<PostSummary<'a>>
}

impl<'a> From<&'a website_new::OrgFile> for PostSummary<'a> {
    fn from(post: &'a website_new::OrgFile) -> Self {
        PostSummary {
            heading: post.title(),
            id: post.id(),
            published: post.published,
            summary: post.from_preamble("summary")
        }
    }
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
            website_name: SerializedLink::from_blog_element(website, website, mode),
            base_url: mode.base_url()
        }
    }
}

impl website_new::Website {
    pub fn serialize<'a, T: Mode>(&self, mode: &T, layout: &'a LayoutInfo) -> SerializedPost<'a> {
        self.page_by_id("about").unwrap().serialize(mode, layout)
    }
}

impl website_new::Project {
    pub fn serialize<'a, T: Mode>(&'a self, _mode: &T, layout: &'a LayoutInfo) -> SerializedProjectIndex<'a> {
        // https://rust-lang-nursery.github.io/rust-cookbook/algorithms/sorting.html
        SerializedProjectIndex {
            layout,
            title: self.title().to_string() + " | Johannes Huwald",
            heading: self.title().to_string(),
            posts: self.posts.values().map(|p| p.into()).collect()
        }
    }
}

impl website_new::OrgFile {
    pub fn serialize<'a, T: Mode>(&self, mode: &T, layout: &'a LayoutInfo) -> SerializedPost<'a> {
        SerializedPost { layout }
    }
}
