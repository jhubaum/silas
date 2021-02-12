use super::website::{Website, Post};
use super::GenerationError;
use serde::Serialize;

use std::fs;
use std::fs::File;

/// the central struct for post serialization and rendering
#[derive(Default)]
pub struct RenderContext<'a> {
    website: Option<&'a Website>,
    post: Option<&'a Post>
}

#[derive(Serialize)]
pub struct SerializedPost {
    #[serde(skip_serializing_if = "Option::is_none")]
    published: Option<String>,
    #[serde(rename = "last-edit")]
    #[serde(skip_serializing_if = "Option::is_none")]
    last_edit: Option<String>,
    content: String,
    title: String,
    heading: String,
    css: Vec<String>
}

impl<'a> RenderContext<'a> {
    pub fn new(website: &'a Website) -> Self {
        RenderContext { website: Some(website), post: None }
    }

    pub fn set_target(&mut self, post: &'a Post) {
       self.post = Some(post);
    }

    pub fn create_file(&self, basepath: &str) -> Result<File, GenerationError> {
        let filename = self.url(basepath.to_string());
        if fs::metadata(&filename).is_ok() {
            return Err(GenerationError::Duplicate);
        }
        println!("Rendering {}", filename);
        fs::create_dir_all(&filename)?;
        let filename = filename + "/index.html";
        Ok(File::create(&filename)?)
    }

    pub fn serialize(&self) -> Result<SerializedPost, GenerationError> {
        if self.post.is_none() {
            panic!("Post in RenderContext is uninitalized");
        }

        let post = self.post.unwrap();

        let mut css_args: Vec<String> = vec![self.resolve_css_path("style.css")];
        for css in post.extra_css.iter() {
            css_args.push(self.resolve_css_path(&css));
        }

        Ok(SerializedPost {
            published: post.published.clone(),
            last_edit: post.last_edit.clone(),
            content: post.content(&self)?,
            title: post.title.clone() + " | Johannes Huwald",
            heading: post.title.clone(),
            css: css_args
        })
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

    fn resolve_link(&self, origin: &Post, link: &str) -> String {
        String::from("The next thing to do!")
    }
}
