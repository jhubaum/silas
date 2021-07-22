use serde::ser::Serialize;
use std::fs;
use std::fs::File;
use std::io::Error as IOError;
use std::path::PathBuf;

mod rendering;
mod rss;
mod serialize;
mod theme;
mod website;

use serialize::LayoutInfo;
use theme::{Theme, ThemeError};
use website::{BlogElement, LoadError, OrgFile, Website};

#[derive(Debug)]
pub enum InitError {
    Theme(ThemeError),
    Website(LoadError),
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

#[derive(Debug)]
pub enum RenderError {
    Theme(theme::RenderError),
    HTML(rendering::HTMLExportError),
    IO(IOError),
    RSS(rss::Error),
}

impl From<theme::RenderError> for RenderError {
    fn from(err: theme::RenderError) -> Self {
        Self::Theme(err)
    }
}

impl From<rendering::HTMLExportError> for RenderError {
    fn from(err: rendering::HTMLExportError) -> Self {
        Self::HTML(err)
    }
}

impl From<IOError> for RenderError {
    fn from(err: IOError) -> Self {
        Self::IO(err)
    }
}

impl From<rss::Error> for RenderError {
    fn from(err: rss::Error) -> Self {
        Self::RSS(err)
    }
}

pub struct Builder<'a> {
    theme: Theme<'a>,
    website: Website,
}

pub trait Mode: Sized {
    fn create(output_path: &str) -> Self;
    fn base_url(&self) -> String;

    fn include_page(&self, page: &OrgFile) -> bool;
    fn include_post(&self, post: &OrgFile) -> bool;
    fn include_rss(&self) -> bool;
}

pub struct ReleaseMode {}
pub struct PreviewMode {
    path: String,
}

impl Mode for ReleaseMode {
    fn create(_output_path: &str) -> Self {
        Self {}
    }

    fn base_url(&self) -> String {
        String::from("https://jhuwald.com")
    }

    fn include_page(&self, page: &OrgFile) -> bool {
        page.published.is_some()
    }

    fn include_post(&self, post: &OrgFile) -> bool {
        if post.published.is_none() {
            return false;
        }
        assert!(
            post.from_preamble("summary").is_some(),
            "published post {:?} is missing a summary",
            post.path
        );
        assert!(
            post.from_preamble("subtitle").is_some(),
            "published post {:?} is missing a summary",
            post.path
        );
        return true;
    }

    fn include_rss(&self) -> bool {
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

    fn include_page(&self, _page: &OrgFile) -> bool {
        true
    }

    fn include_post(&self, post: &OrgFile) -> bool {
        if post.published.is_some() {
            if post.from_preamble("summary").is_none() {
                println!("published post {:?} is missing a summary", post.path);
            }
            if post.from_preamble("subtitle").is_none() {
                println!("published post {:?} is missing a subtitle", post.path);
            }
        }
        true
    }

    fn include_rss(&self) -> bool {
        false
    }
}

impl Builder<'_> {
    pub fn new(website_path: &str) -> Result<Self, InitError> {
        Ok(Builder {
            theme: Theme::load("./theme")?,
            website: Website::load(website_path)?,
        })
    }

    pub fn generate<TMode: Mode>(
        &self,
        output_path: &str,
        overwrite_existing: bool,
    ) -> Result<(), RenderError> {
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
        let mut rss = rss::RSSBuilder::new(&self.website, &mode);

        let mut ser = self.website.serialize(&mode, &layout)?;
        let file = self.prepare_file(&self.website, output_path, &mut ser.folder_out)?;
        self.render_element(file, "page", &ser)?;

        for page in self.website.pages.values() {
            if !mode.include_page(&page) {
                continue;
            }
            let mut ser = page.serialize(&self.website, &mode, &layout)?;
            rss.insert_file(&ser);
            let file = self.prepare_file(page, output_path, &mut ser.folder_out)?;
            self.render_element(file, "page", &ser)?;
        }

        for project in self.website.projects.values() {
            let mut ser = project.serialize(&self.website, &mode, &layout)?;
            rss.start_project(project.id(), &ser);
            let file = self.prepare_file(project, output_path, &mut ser.folder_out)?;
            self.render_element(file, "project", &ser)?;

            for post in project.posts.values() {
                if !mode.include_post(&post) {
                    continue;
                }
                let mut ser = post.serialize(&self.website, &mode, &layout)?;
                rss.insert_file(&ser);
                let file = self.prepare_file(post, output_path, &mut ser.folder_out)?;
                self.render_element(file, "post", &ser)?;
            }
            rss.finish_project();
        }

        rss.write_feeds(output_path)?;

        Ok(())
    }

    fn render_element<T: Serialize>(
        &self,
        mut file: File,
        template: &str,
        elem: &serialize::SerializedResult<T>,
    ) -> Result<(), RenderError> {
        for img in elem.image_deps.iter() {
            fs::copy(
                elem.folder_in.clone() + "/" + img,
                elem.folder_out.clone() + "/" + img,
            )?;
        }
        self.theme.render(&mut file, template, &elem.elem)?;
        Ok(())
    }

    fn prepare_file<T: BlogElement>(
        &self,
        elem: &T,
        output_path: &str,
        folder_out: &mut String,
    ) -> Result<File, IOError> {
        let filename = elem.url(&self.website, output_path.to_string());
        fs::create_dir_all(&filename)?;
        *folder_out = filename.clone();
        let filename = filename + "/index.html";
        Ok(File::create(&filename)?)
    }
}
