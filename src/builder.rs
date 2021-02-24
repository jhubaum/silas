use std::collections::BinaryHeap;

use std::path::{Path, PathBuf};
use std::io::{Error as IOError};
use std::string::FromUtf8Error;
use std::fs;
use std::fs::File;

use rss::{ChannelBuilder, ItemBuilder};

use serde::ser::{Serialize, Serializer, SerializeStruct};

mod theme;
use theme::{Theme, ThemeError, RenderError};

pub mod website_new;
use website_new::{Website, LoadError, BlogElement, OrgFile};

mod serialize;
use serialize::LayoutInfo;

#[derive(Debug)]
pub enum InitError {
    Theme(ThemeError),
    Website(LoadError)
}

impl From<ThemeError> for InitError {
    fn from(err: ThemeError) -> Self {
        Self::Theme(err)
    }
}

impl From<LoadError> for InitError {
    fn from(err: LoadError) -> Self {
        Self::Website(err)
    }
}

pub enum PostStatus<'a> {
    Ok, // everything's ok
    Ignore, // everything's ok, don't include this post in rendering
    Warning(&'a str), // use this post, but print a warning
    Error(&'a str) // critical error. Cancel rendering
}

impl PostStatus<'_> {
    fn resolve(&self, post_type: &str, post_id: &str) -> bool {
        match self {
            PostStatus::Warning(warn) => {
                println!("Warning for {} '{}': {}", post_type,
                         post_id, warn);
                true
            },
            PostStatus::Error(err) => {
                panic!("Error for {} '{}': {}", post_type, post_id, err);
            },
            PostStatus::Ok => true,
            PostStatus::Ignore => false
        }
    }
}

pub struct Builder<'a> {
    theme: Theme<'a>,
    website: Website
}

pub trait Mode: Sized {
    fn create(output_path: &str) -> Self;
    fn base_url(&self) -> String;

    fn include_page(&self, page: &OrgFile) -> bool;
    fn include_post(&self, post: &OrgFile) -> bool;
}

pub struct ReleaseMode {  }
pub struct PreviewMode {
    path: String,
}

impl Mode for ReleaseMode {
    fn create(_output_path: &str) -> Self {
        Self {  }
    }

    fn base_url(&self) -> String {
        String::from("https://jhuwald.com")
    }

    fn include_page(&self, page: &OrgFile) -> bool {
        true
    }

    fn include_post(&self, post: &OrgFile) -> bool {
        true
    }
}

impl Mode for PreviewMode {
    fn create(output_path: &str) -> Self {
        let path = PathBuf::from(output_path).canonicalize().unwrap();
        let path = path.to_str().unwrap().to_string();

        Self { path }
    }

    fn base_url(&self) -> String {
        self.path.clone()
    }

    fn include_page(&self, page: &OrgFile) -> bool {
        true
    }

    fn include_post(&self, post: &OrgFile) -> bool {
        true
    }
}

impl Builder<'_> {
    pub fn new(website_path: &str) -> Result<Self, InitError> {
        Ok(Builder {
            theme: Theme::load("./theme")?,
            website: Website::load(website_path)?
        })
    }

    pub fn generate<TMode: Mode>(&self, output_path: &str, overwrite_existing: bool) -> Result<(), RenderError> {
        if fs::metadata(output_path).is_ok() {
            if !overwrite_existing {
                panic!("Target folder '{}' is non-empty", output_path);
            }

            println!("Cleared previous result");
            fs::remove_dir_all(output_path)?;
        }
        fs::create_dir(output_path)?;
        self.theme.copy_files(output_path)?;

        let mode = TMode::create(output_path);
        let layout = LayoutInfo::new(&self.website, &mode);

        let mut file = self.prepare_file(&self.website, output_path)?;
        self.theme.render(&mut file, "page",
                          &self.website.serialize(&mode, &layout))?;

        for page in self.website.pages.values() {
            let mut file = self.prepare_file(page, output_path)?;
            self.theme.render(&mut file, "page",
                              &page.serialize(&mode, &layout))?;
        }

        for project in self.website.projects.values() {
            let mut file = self.prepare_file(project, output_path)?;
            self.theme.render(&mut file, "project",
                              &project.serialize(&mode, &layout))?;

            for post in project.posts.values() {
                let mut file = self.prepare_file(post, output_path)?;
                self.theme.render(&mut file, "post",
                                  &post.serialize(&mode, &layout))?;
            }
        }

        Ok(())
    }

    fn prepare_file<T: BlogElement>(&self, elem: &T, output_path: &str) -> Result<File, IOError> {
        let filename = elem.url(&self.website, output_path.to_string());
        fs::create_dir_all(&filename)?;
        let filename = filename + "/index.html";
        Ok(File::create(&filename)?)
    }
}
