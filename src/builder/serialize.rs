use super::rendering;
use super::rendering::OrgExtractGenerator;
use super::website;
use super::website::{BlogElement, PostOrder};
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

#[derive(PartialOrd, PartialEq, Eq, Ord)]
enum LinkType {
    WebsiteIndex = 0,
    Page = 1,
    Project = 2,
}

#[derive(Serialize)]
pub struct SerializedLink {
    target: String,
    title: String,
    #[serde(skip_serializing)]
    // this enum is only used to sort the list of links
    link_type: LinkType,
}

#[derive(Serialize)]
pub struct SerializedPost<'a> {
    layout: &'a LayoutInfo,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub published: Option<chrono::naive::NaiveDate>,
    #[serde(rename = "last-edit")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_edit: Option<chrono::naive::NaiveDate>,
    pub content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subtitle: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary: Option<String>,
    pub title: String,
    pub heading: &'a str,
    pub id: &'a str,
}

pub struct SerializedResult<T: Serialize> {
    pub elem: T,
    pub image_deps: Vec<String>,
    pub folder_in: String,
    pub folder_out: String,
    pub url: String,
}

#[derive(Serialize)]
struct PostSummary<'a> {
    heading: &'a str,
    id: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    published: Option<chrono::naive::NaiveDate>,
    #[serde(skip_serializing_if = "Option::is_none")]
    subtitle: Option<&'a str>,
}

#[derive(Serialize)]
pub struct SerializedProjectIndex<'a> {
    layout: &'a LayoutInfo,
    pub title: String,
    pub heading: String,
    pub description: &'a str,
    pub text: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    published: Option<chrono::naive::NaiveDate>,
    posts: Vec<PostSummary<'a>>,
}

impl<'a> From<&'a website::OrgFile> for PostSummary<'a> {
    fn from(post: &'a website::OrgFile) -> Self {
        PostSummary {
            heading: post.title(),
            id: post.id(),
            published: post.published,
            subtitle: post.from_preamble("subtitle"),
        }
    }
}

impl SerializedLink {
    fn from_blog_element<TElem: BlogElement, TMode: Mode>(
        elem: &TElem,
        website: &website::Website,
        mode: &TMode,
        link_type: LinkType,
    ) -> Self {
        SerializedLink {
            target: elem.url(website, mode.base_url()),
            title: elem.title().to_string(),
            link_type,
        }
    }
}

impl LayoutInfo {
    pub fn new<T: Mode>(website: &website::Website, mode: &T) -> Self {
        let mut header = Vec::new();
        for page in website.pages.values() {
            if mode.include_page(page) {
                let link = SerializedLink::from_blog_element(page, website, mode, LinkType::Page);
                header.push(link);
            }
        }

        for proj in website.projects.values() {
            if mode.include_project(proj) {
                let link =
                    SerializedLink::from_blog_element(proj, website, mode, LinkType::Project);
                header.push(link);
            }
        }

        header.sort_by(|lhs, rhs| {
            let cmp = lhs.link_type.cmp(&rhs.link_type);
            if cmp == std::cmp::Ordering::Equal {
                lhs.title.cmp(&rhs.title)
            } else {
                cmp
            }
        });

        LayoutInfo {
            header,
            website_name: SerializedLink::from_blog_element(
                website,
                website,
                mode,
                LinkType::WebsiteIndex,
            ),
            base_url: mode.base_url(),
        }
    }
}

impl website::Website {
    pub fn serialize<'a, T: Mode>(
        &'a self,
        mode: &T,
        layout: &'a LayoutInfo,
    ) -> Result<SerializedResult<SerializedPost<'a>>, rendering::HTMLExportError> {
        self.index.serialize(self, mode, layout)
    }
}

fn sort_by_published(lhs: &PostSummary, rhs: &PostSummary) -> std::cmp::Ordering {
    match lhs.published {
        None => std::cmp::Ordering::Less,
        Some(lhs) => {
            if rhs.published.is_none() {
                std::cmp::Ordering::Greater
            } else {
                lhs.cmp(&rhs.published.unwrap()).reverse()
            }
        }
    }
}

fn sort_by_id(lhs: &PostSummary, rhs: &PostSummary) -> std::cmp::Ordering {
    lhs.id.cmp(rhs.id)
}

impl website::Project {
    pub fn serialize<'a, T: Mode>(
        &'a self,
        website: &'a website::Website,
        mode: &T,
        layout: &'a LayoutInfo,
    ) -> Result<SerializedResult<SerializedProjectIndex<'a>>, rendering::HTMLExportError> {
        let mut posts: Vec<PostSummary> = self
            .posts
            .values()
            .filter(|p| mode.include_post(&p))
            .map(|p| p.into())
            .collect();

        match self
            .index
            .parse_from_preamble::<PostOrder>("order")
            .unwrap_or_default()
        {
            PostOrder::NewestFirst => posts.sort_by(sort_by_published),
            PostOrder::ById => posts.sort_by(sort_by_id),
        };
        let index = self.index.serialize(website, mode, layout)?;
        Ok(SerializedResult {
            elem: SerializedProjectIndex {
                layout,
                posts,
                text: index.elem.content,
                title: self.title().to_string() + " | Johannes Huwald",
                heading: self.title().to_string(),
                description: self.description(),
                published: self.index.published,
            },
            image_deps: index.image_deps,
            folder_in: index.folder_in,
            folder_out: index.folder_out,
            url: self.url(&website, mode.base_url()),
        })
    }
}

impl website::OrgFile {
    pub fn serialize<'a, T: Mode>(
        &'a self,
        website: &'a website::Website,
        mode: &T,
        layout: &'a LayoutInfo,
    ) -> Result<SerializedResult<SerializedPost<'a>>, rendering::HTMLExportError> {
        let rr = self.render_html(website, mode)?;
        let mut folder_in = self.path.clone();
        folder_in.pop();
        let folder_in = folder_in.to_str().unwrap().to_string();

        let mut summary = self.parse_from_preamble::<String>("summary");

        if summary.is_none() && self.post_type == website::PostType::Mini {
            summary = Some(OrgExtractGenerator::generate(self)?);
        }

        Ok(SerializedResult {
            image_deps: rr.image_deps,
            folder_in,
            folder_out: String::new(),
            url: self.url(&website, mode.base_url()),
            elem: SerializedPost {
                layout,
                summary,
                published: self.published,
                last_edit: self.last_edit,
                content: rr.content,
                subtitle: self.from_preamble("subtitle"),
                title: self.title().to_string() + " | Johannes Huwald",
                heading: self.title(),
                id: self.id(),
            },
        })
    }
}
