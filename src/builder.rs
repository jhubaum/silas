use serde::ser::Serialize;
use std::fs;
use std::fs::File;
use std::io::Error as IOError;
use std::path::{Path, PathBuf};

mod fileutil;
mod rendering;
mod rss;
mod serialize;
mod theme;
mod website;

use serialize::LayoutInfo;
use theme::{TemplateType, Theme, ThemeError};
use website::{BlogElement, OrgFile, Project, Website, WebsiteError};

#[derive(Debug)]
pub enum InitError {
    Theme(ThemeError),
    Website(WebsiteError),
    IO(IOError),
}

impl From<ThemeError> for InitError {
    fn from(err: ThemeError) -> Self {
        Self::Theme(err)
    }
}

impl From<WebsiteError> for InitError {
    fn from(err: WebsiteError) -> Self {
        Self::Website(err)
    }
}

impl From<IOError> for InitError {
    fn from(err: IOError) -> Self {
        Self::IO(err)
    }
}

#[derive(Debug)]
pub enum RenderError {
    Theme(theme::RenderError),
    HTML(rendering::SerializationError),
    IO(IOError),
    RSS(rss::Error),
    FileNotFound(String),
    InvalidImageDependency { file: String, dependency: String },
}

impl From<theme::RenderError> for RenderError {
    fn from(err: theme::RenderError) -> Self {
        Self::Theme(err)
    }
}

impl From<rendering::SerializationError> for RenderError {
    fn from(err: rendering::SerializationError) -> Self {
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
    temp_dir: PathBuf,
    output_path: &'a str,
}

pub trait Mode: Sized {
    fn create(builder: &Builder) -> Self;
    fn base_url(&self) -> String;

    fn include_page(page: &OrgFile) -> Result<bool, website::OrgFileError>;
    fn include_post(post: &OrgFile) -> Result<bool, website::OrgFileError>;
    fn include_project(project: &Project) -> Result<bool, website::ProjectError>;
    fn include_rss() -> bool;
}

pub struct ReleaseMode {}
pub struct PreviewMode {
    path: String,
}

impl Mode for ReleaseMode {
    fn create(_: &Builder) -> Self {
        Self {}
    }

    fn base_url(&self) -> String {
        String::from("https://jhuwald.com")
    }

    fn include_page(page: &OrgFile) -> Result<bool, website::OrgFileError> {
        Ok(page.published.is_some())
    }

    fn include_post(post: &OrgFile) -> Result<bool, website::OrgFileError> {
        if post.published.is_none() {
            return Ok(false);
        }

        if post.post_type == website::PostType::Mini {
            return Ok(true);
        }

        if post.from_preamble("summary").is_none() {
            return Err(website::OrgFileError::MissingRequiredField("summary"));
        }

        if post.from_preamble("subtitle").is_none() {
            return Err(website::OrgFileError::MissingRequiredField("subtitle"));
        }

        return Ok(true);
    }

    fn include_project(project: &Project) -> Result<bool, website::ProjectError> {
        Ok(project.published())
    }

    fn include_rss() -> bool {
        true
    }
}

impl Mode for PreviewMode {
    fn create(builder: &Builder) -> Self {
        let path = PathBuf::from(builder.output_path).canonicalize().unwrap();
        Self {
            path: path.to_str().unwrap().to_string(),
        }
    }

    fn base_url(&self) -> String {
        self.path.clone()
    }

    fn include_page(_: &OrgFile) -> Result<bool, website::OrgFileError> {
        Ok(true)
    }

    fn include_post(post: &OrgFile) -> Result<bool, website::OrgFileError> {
        if let Err(err) = ReleaseMode::include_post(post) {
            println!("Warning: {:?} in {:?}", err, post.path);
        }
        Ok(true)
    }

    fn include_project(_: &Project) -> Result<bool, website::ProjectError> {
        Ok(true)
    }

    fn include_rss() -> bool {
        false
    }
}

impl<'a> Builder<'a> {
    pub fn new<TMode: Mode>(
        website_path: &str,
        theme_path: &str,
        output_path: &'a str,
    ) -> Result<Self, InitError> {
        let theme = Theme::load(theme_path)?;
        let website = Website::load::<TMode>(website_path)?;

        let mut temp_dir = std::env::temp_dir();
        temp_dir.push("silas-generated-output");

        assert!(
            !temp_dir.is_dir(),
            "Silas: temporary directory already exists, aborting generation."
        );
        fs::create_dir(&temp_dir)?;

        Ok(Builder {
            theme,
            website,
            temp_dir,
            output_path,
        })
    }

    pub fn copy_generated_files(&self) -> Result<(), IOError> {
        if fs::metadata(self.output_path).is_ok() {
            // clear previous result
            fs::remove_dir_all(self.output_path)?;
        }

        let res = fileutil::copy_folder_recursively(&self.temp_dir, self.output_path);
        self.clear_generated_files();
        res
    }

    pub fn clear_generated_files(&self) {
        fs::remove_dir_all(&self.temp_dir).unwrap();
    }

    pub fn generate_single_file<TMode: Mode>(&self, file_path: &str) -> Result<(), RenderError> {
        let mode = TMode::create(self);
        let layout = LayoutInfo::new(&self.website, &mode);

        match self.website.resolve_path(Path::new(file_path)) {
            None => Err(RenderError::FileNotFound(file_path.to_string())),
            Some(post) => {
                let ser = post.serialize(&self.website, &mode, &layout)?;
                let file =
                    File::create(self.temp_dir.to_str().unwrap().to_string() + "/index.html")?;
                self.render_element(file, TemplateType::Post, &ser)?;
                Ok(())
            }
        }
    }

    pub fn generate<TMode: Mode>(&self) -> Result<(), RenderError> {
        let mode = TMode::create(self);

        self.theme.copy_files(self.temp_dir.to_str().unwrap())?;

        let layout = LayoutInfo::new(&self.website, &mode);
        let mut rss = rss::RSSBuilder::new(&self.website, &mode);

        let mut ser = self.website.serialize(&mode, &layout)?;
        let file = self.prepare_file(&self.website, &mut ser.folder_out)?;
        self.render_element(file, TemplateType::Page, &ser)?;
        rss.insert_file(&ser);

        for page in self.website.pages.values() {
            let mut ser = page.serialize(&self.website, &mode, &layout)?;
            rss.insert_file(&ser);
            let file = self.prepare_file(page, &mut ser.folder_out)?;
            self.render_element(file, TemplateType::Page, &ser)?;
        }

        for project in self.website.projects.values() {
            let mut ser = project.serialize(&self.website, &mode, &layout)?;
            rss.start_project(project.id(), &ser);
            let file = self.prepare_file(project, &mut ser.folder_out)?;
            self.render_element(file, TemplateType::Project(project.project_type), &ser)?;

            for post in project.posts.values() {
                let mut ser = post.serialize(&self.website, &mode, &layout)?;
                rss.insert_file(&ser);
                let file = self.prepare_file(post, &mut ser.folder_out)?;
                self.render_element(file, TemplateType::Post, &ser)?;
            }
            rss.finish_project();
        }

        rss.write_feeds(self.temp_dir.to_str().unwrap())?;
        Ok(())
    }

    fn render_element<T: Serialize>(
        &self,
        mut file: File,
        template: TemplateType,
        elem: &serialize::SerializedResult<T>,
    ) -> Result<(), RenderError> {
        for img in elem.image_deps.iter() {
            let mut path = PathBuf::from(&elem.folder_in);
            path.push(img);
            if !path.is_file() {
                return Err(RenderError::InvalidImageDependency {
                    file: elem.url.clone(),
                    dependency: path.to_str().unwrap().to_string(),
                });
            }
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
        folder_out: &mut String,
    ) -> Result<File, IOError> {
        let filename = self.temp_dir.to_str().unwrap().to_string();
        let filename = elem.url(&self.website, filename);
        fs::create_dir_all(&filename)?;
        *folder_out = filename.clone();
        let filename = filename + "/index.html";
        Ok(File::create(&filename)?)
    }
}

#[test]
fn test_release_mode() -> Result<(), WebsiteError>{
    let website = Website::load::<ReleaseMode>("testsite")?;
    assert!(website.page_by_id("unpublished").is_none());

    Ok(())
}

#[test]
fn test_preview_mode() -> Result<(), WebsiteError>{
    let website = Website::load::<PreviewMode>("testsite")?;

    let unpub = website.page_by_id("unpublished");
    assert!(unpub.is_some());
    let unpub = unpub.unwrap();
    assert!(unpub.published.is_none());

    Ok(())
}
