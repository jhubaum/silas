use super::serialize::{SerializedPost, SerializedProjectIndex, SerializedResult};
use super::website::{BlogElement, Website};
use super::Mode;
use chrono::prelude::*;
use std::collections::HashMap;
use std::fs::File;
use std::io::Error as IOError;

use rss::{Channel, ChannelBuilder, Item, ItemBuilder};

pub struct RSSBuilder<'a> {
    current_project: Option<&'a str>,
    website: Channel,
    projects: HashMap<&'a str, Channel>,
    last_build: String,
}

#[derive(Debug)]
pub enum Error {
    RSS(rss::Error),
    IO(IOError),
}

impl From<rss::Error> for Error {
    fn from(err: rss::Error) -> Self {
        Self::RSS(err)
    }
}

impl From<IOError> for Error {
    fn from(err: IOError) -> Self {
        Self::IO(err)
    }
}

type Post<'a> = SerializedResult<SerializedPost<'a>>;

impl<'a> From<&Post<'a>> for Item {
    fn from(post: &Post) -> Self {
        let mut builder = ItemBuilder::default();
        builder
            .title(post.elem.heading.to_string())
            .link(post.url.to_string())
            .author(String::from("Johannes Huwald <hey@jhuwald.com>"))
            .content(post.elem.content.to_string());

        if let Some(date) = post.elem.published {
            let date = Local.from_local_date(&date).unwrap().and_hms(10, 0, 0);
            builder.pub_date(date.to_rfc2822());
        }

        if let Some(desc) = post.elem.summary {
            builder.description(desc.to_string());
        };
        builder.build().unwrap()
    }
}

impl<'a> RSSBuilder<'a> {
    pub fn new<TMode: Mode>(website: &Website, mode: &TMode) -> Self {
        let projects = HashMap::new();
        let local: DateTime<Local> = Local::now();
        let last_build = local.to_rfc2822();

        let website_channel = ChannelBuilder::default()
            .title(website.title())
            .link(website.url(&website, mode.base_url()))
            .description(website.description())
            .managing_editor(String::from("hey@jhuwald.com"))
            .webmaster(String::from("hey@jhuwald.com"))
            .last_build_date(last_build.clone())
            .build()
            .unwrap();

        RSSBuilder {
            projects,
            last_build,
            website: website_channel,
            current_project: None,
        }
    }

    pub fn write_feeds(&self, path: &str) -> Result<(), Error> {
        let file = File::create(&(path.to_string() + "/feed"))?;
        self.website.write_to(file)?;

        for p in self.projects.iter() {
            let file = File::create(&(path.to_string() + "/" + p.0 + "/feed"))?;
            p.1.write_to(file)?;
        }

        Ok(())
    }

    pub fn insert_file(&mut self, file: &Post) {
        if let Some(id) = self.current_project {
            self.projects.get_mut(&id).unwrap().items.push(file.into());
        }
        self.website.items.push(file.into());
    }

    pub fn start_project(
        &mut self,
        id: &'a str,
        project: &SerializedResult<SerializedProjectIndex>,
    ) {
        let channel = ChannelBuilder::default()
            .title(project.elem.title.to_string())
            .link(project.url.to_string())
            .description(project.elem.description.to_string())
            .managing_editor(String::from("hey@jhuwald.com"))
            .webmaster(String::from("hey@jhuwald.com"))
            .last_build_date(self.last_build.clone())
            .build()
            .unwrap();
        self.projects.insert(id, channel);
        self.current_project = Some(id);
    }

    pub fn finish_project(&mut self) {
        self.current_project = None;
    }
}
