use super::rendering;
use super::website_new;
use super::website_new::BlogElement;
use super::Mode;
use serde::Serialize;

#[derive(Serialize)]
pub struct LayoutInfo {
    header: Vec<SerializedLink>,
    #[serde(rename = "website-name")]
    website_name: SerializedLink,
    #[serde(rename = "base-url")]
    base_url: String,
}

#[derive(Serialize)]
pub struct SerializedLink {
    target: String,
    title: String,
}

#[derive(Serialize)]
pub struct SerializedPost<'a> {
    layout: &'a LayoutInfo,
    #[serde(skip_serializing_if = "Option::is_none")]
    published: Option<chrono::naive::NaiveDate>,
    #[serde(rename = "last-edit")]
    #[serde(skip_serializing_if = "Option::is_none")]
    last_edit: Option<chrono::naive::NaiveDate>,
    content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    summary: Option<&'a str>,
    title: String,
    heading: &'a str,
    id: &'a str,
}

pub struct SerializedResult<T: Serialize> {
    pub elem: T,
    pub image_deps: Vec<String>,
    pub folder_in: String,
    pub folder_out: String,
}

impl<T: Serialize> SerializedResult<T> {
    fn no_deps(elem: T) -> Self {
        Self {
            elem,
            image_deps: Vec::new(),
            folder_in: String::new(),
            folder_out: String::new(),
        }
    }
}

#[derive(Serialize)]
struct PostSummary<'a> {
    heading: &'a str,
    id: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    published: Option<chrono::naive::NaiveDate>,
    #[serde(skip_serializing_if = "Option::is_none")]
    summary: Option<&'a str>,
}

#[derive(Serialize)]
pub struct SerializedProjectIndex<'a> {
    layout: &'a LayoutInfo,
    title: String,
    heading: String,
    posts: Vec<PostSummary<'a>>,
}

impl<'a> From<&'a website_new::OrgFile> for PostSummary<'a> {
    fn from(post: &'a website_new::OrgFile) -> Self {
        PostSummary {
            heading: post.title(),
            id: post.id(),
            published: post.published,
            summary: post.from_preamble("summary"),
        }
    }
}

impl SerializedLink {
    fn from_blog_element<TElem: BlogElement, TMode: Mode>(
        elem: &TElem,
        website: &website_new::Website,
        mode: &TMode,
    ) -> Self {
        SerializedLink {
            target: elem.url(website, mode.base_url()),
            title: elem.title().to_string(),
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
            base_url: mode.base_url(),
        }
    }
}

impl website_new::Website {
    pub fn serialize<'a, T: Mode>(
        &'a self,
        mode: &T,
        layout: &'a LayoutInfo,
    ) -> Result<SerializedResult<SerializedPost<'a>>, rendering::HTMLExportError> {
        self.page_by_id("about")
            .unwrap()
            .serialize(self, mode, layout)
    }
}

impl website_new::Project {
    pub fn serialize<'a, T: Mode>(
        &'a self,
        mode: &T,
        layout: &'a LayoutInfo,
    ) -> SerializedResult<SerializedProjectIndex<'a>> {
        let mut posts: Vec<PostSummary> = self
            .posts
            .values()
            .filter(|p| mode.include_post(&p))
            .map(|p| p.into())
            .collect();

        posts.sort_by(|a, b| match a.published {
            None => std::cmp::Ordering::Less,
            Some(a) => {
                if b.published.is_none() {
                    std::cmp::Ordering::Greater
                } else {
                    a.cmp(&b.published.unwrap()).reverse()
                }
            }
        });
        SerializedResult::no_deps(SerializedProjectIndex {
            layout,
            posts,
            title: self.title().to_string() + " | Johannes Huwald",
            heading: self.title().to_string(),
        })
    }
}

impl website_new::OrgFile {
    pub fn serialize<'a, T: Mode>(
        &'a self,
        website: &'a website_new::Website,
        mode: &T,
        layout: &'a LayoutInfo,
    ) -> Result<SerializedResult<SerializedPost<'a>>, rendering::HTMLExportError> {
        let rr = self.render_html(website, mode)?;
        let mut folder_in = self.path.clone();
        folder_in.pop();
        let folder_in = folder_in.to_str().unwrap().to_string();
        Ok(SerializedResult {
            image_deps: rr.image_deps,
            folder_in,
            folder_out: String::new(),
            elem: SerializedPost {
                layout,
                published: self.published,
                last_edit: self.last_edit,
                content: rr.content,
                summary: self.from_preamble("summary"),
                title: self.title().to_string() + " | Johannes Huwald",
                heading: self.title(),
                id: self.id(),
            },
        })
    }
}
