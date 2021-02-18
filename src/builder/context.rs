use super::website::{Website, Post};
use super::GenerationError;
use serde::Serialize;
use std::cmp::Ordering;

use std::cell::RefCell;

use std::fs;
use std::fs::File;

/// the central struct for post serialization and rendering
pub struct RenderContext<'a> {
    pub website: Option<&'a Website>,
    pub post: Option<&'a Post>,
    pub folder_out: &'a str,
    image_deps: RefCell<Vec<String>>
}

#[derive(Serialize, Clone, PartialEq, Eq)]
pub struct LayoutInfo {
    pub header: Vec<SerializedLink>,
    #[serde(rename = "base-url")]
    pub base_url: String
}

impl LayoutInfo {
    pub fn new(url: String) -> Self {
        LayoutInfo { header: Vec::new(), base_url: url }
    }

    pub fn insert_header(&mut self, target: &str, title: String) {
        let link = SerializedLink {
            target: self.base_url.clone() + "/" + target,
            title
        };
        self.header.push(link);
    }

    pub fn insert_post_in_header(&mut self, post: &Post) {
        self.insert_header(post.id(), post.title.clone());
    }
}

#[derive(Serialize, Clone, PartialEq, Eq)]
pub struct SerializedLink {
    target: String,
    title: String
}

pub enum ResolvedInternalLink {
    Post(String),
    Image(String)
}

#[derive(Serialize, Eq)]
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
    heading: String,
    css: Vec<String>,
    id: String
}

impl Ord for SerializedPost<'_> {
    fn cmp(&self, rhs: &Self) -> Ordering {
        match self.published {
            None => { return if rhs.published.is_none() {Ordering::Equal} else {Ordering::Greater} },
            Some(d) => { return if rhs.published.is_none() {Ordering::Less} else {d.cmp(&rhs.published.unwrap())} }
        }
    }
}

impl PartialOrd for SerializedPost<'_> {
    fn partial_cmp(&self, rhs: &Self) -> Option<Ordering> {
        Some(self.cmp(rhs))
    }
}

impl PartialEq for SerializedPost<'_> {
    fn eq(&self, rhs: &Self) -> bool {
        self.published == rhs.published
    }
}


impl Default for RenderContext<'_> {
    fn default() -> Self {
        RenderContext { website: None, post: None,
                        folder_out: ".", image_deps: RefCell::new(Vec::new()) }
    }
}

impl<'a> RenderContext<'a> {
    pub fn new(website: &'a Website, folder_out: &'a str) -> Self {
        RenderContext { website: Some(website), post: None,
                        folder_out, image_deps: RefCell::new(Vec::new()) }
    }

    pub fn set_target(&mut self, post: &'a Post) {
        self.post = Some(post);
        self.image_deps.borrow_mut().clear();
    }

    pub fn create_file(&self, basepath: &str) -> Result<File, GenerationError> {
        let filename = self.url(basepath.to_string());
        if fs::metadata(&filename).is_ok() {
            return Err(GenerationError::Duplicate);
        }
        fs::create_dir_all(&filename)?;
        let filename = filename + "/index.html";
        Ok(File::create(&filename)?)
    }

    pub fn serialize(&self, layout: &'a LayoutInfo) -> Result<SerializedPost<'a>, GenerationError> {
        if self.post.is_none() {
            panic!("Post in RenderContext is uninitalized");
        }

        let post = self.post.unwrap();

        let mut css_args: Vec<String> = vec![self.resolve_css_path("style.css")];
        for css in post.extra_css.iter() {
            css_args.push(self.resolve_css_path(&css));
        }

        Ok(SerializedPost {
            layout,
            published: post.published,
            last_edit: post.last_edit,
            summary: post.summary(),
            content: post.content(&self)?,
            title: post.title.clone() + " | Johannes Huwald",
            heading: post.title.clone(),
            css: css_args,
            id: post.id().to_string()
        })
    }

    pub fn copy_images(&self) -> Result<(), GenerationError> {
        for img in self.image_deps.borrow().iter() {
            let origin = self.post.unwrap().path
                             .parent().unwrap()
                             .to_str().unwrap().to_string() + "/" + &img;
            println!("Origin: {:?}", origin);
            fs::copy(origin,
                     self.url(self.folder_out.to_string()) + "/" + &img)?;
        }
        Ok(())
    }

    fn resolve_css_path(&self, filename: &str) -> String {
        match self.post.unwrap().index.project {
            None => String::from("../css/") + filename,
            Some(_) => String::from("../../css/") + filename
        }
    }

    fn url(&self, base: String) -> String {
        let post = &self.post.unwrap();
        match post.index.project {
            None => base + "/" + &post.id(),
            Some(_) => base + "/blog/" + &post.id()
        }
    }

    fn website_url(&self) -> String {
        match self.website {
            None => String::from(""),
            Some(w) => w.url()
        }
    }

    pub fn resolve_link(&self, link: &str) -> Result<ResolvedInternalLink, GenerationError> {
        match link.split(".").last().unwrap() {
            "org" => self.resolve_post_link(link),
            "png" | "jpeg" => self.resolve_image_link(link),
            t => Err(GenerationError::UnknownLinkType(t.to_string()))
        }
    }

    fn resolve_post_link(&self, link: &str) -> Result<ResolvedInternalLink, GenerationError> {
        let other = self.find_post_from_relative_link(link);

        if other.is_none() {
            println!("{} points to no file", link);
            return Err(GenerationError::InvalidLink);
        }

        let link = self.website.unwrap().get_relative_url(
            self.post.unwrap(), other.unwrap()
        );
        Ok(ResolvedInternalLink::Post(link))
    }

    fn find_post_from_relative_link(&self, link: &str) -> Option<&Post> {
        let mut path = self.post.unwrap().path.clone();
        path.pop();
        for part in link.split("/") {
            match part {
                "." => continue,
                ".." => path.pop(),
                _ => { path.push(part); true }
            };
        }

        self.website.unwrap().find_post_from_path(path)
    }

    fn resolve_image_link(&self, link: &str) -> Result<ResolvedInternalLink, GenerationError> {
        self.image_deps.borrow_mut().push(String::from(link));
        Ok(ResolvedInternalLink::Image(String::from(link)))
    }
}
