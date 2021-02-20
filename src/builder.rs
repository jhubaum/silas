use handlebars::{Handlebars, RenderContext, Helper, Context, JsonRender, HelperResult, Output, RenderError, TemplateFileError};

use std::collections::BinaryHeap;

use std::path::{Path, PathBuf};
use std::io::{Error as IOError, Write};
use std::string::FromUtf8Error;
use std::fs;
use std::fs::File;

use rss::{ChannelBuilder, ItemBuilder};

use serde::ser::{Serialize, Serializer, SerializeStruct};

use chrono::naive::NaiveDate;

pub mod website;
pub mod org;
mod context;

use website::{WebsiteLoadError, Website, Post, PostIndex};
use org::OrgLoadError;

fn render_date (h: &Helper, _: &Handlebars, _: &Context, _rc: &mut RenderContext, out: &mut dyn Output) -> HelperResult {
    let param = h.param(0).unwrap();

    let date = param.value().render();
    let date = NaiveDate::parse_from_str(date.as_ref(), "%Y-%m-%d").unwrap();
    out.write(&date.format("%A, %d. %B %Y").to_string())?;
    Ok(())
}

pub enum PostStatus<'a> {
    Ok, // everything's ok
    Ignore, // everything's ok, don't include this post in rendering
    Warning(&'a str), // use this post, but print a warning
    Error(&'a str) // critical error. Cancel rendering
}

#[derive(Debug)]
pub enum InstantiationError {
    TemplateNotFound(TemplateFileError)
}

#[derive(Debug)]
pub enum GenerationError {
    IO(IOError),
    Rendering(RenderError),
    File(WebsiteLoadError),
    Utf8(FromUtf8Error),
    Duplicate,
    InvalidLink,
    UnknownLinkType(String),
    RSS(rss::Error)
}

#[derive(Debug)]
pub enum SingleFileError {
    Instantiation(InstantiationError),
    GenerationError(GenerationError)
}

impl From<TemplateFileError> for InstantiationError {
    fn from(err: TemplateFileError) -> Self {
        InstantiationError::TemplateNotFound(err)
    }
}

impl From<WebsiteLoadError> for GenerationError {
    fn from(err: WebsiteLoadError) -> Self {
        GenerationError::File(err)
    }
}

impl From<OrgLoadError> for GenerationError {
    fn from(err: OrgLoadError) -> Self {
        GenerationError::File(err.into())
    }
}

impl From<IOError> for GenerationError {
    fn from(err: IOError) -> Self {
        GenerationError::IO(err)
    }
}

impl From<RenderError> for GenerationError {
    fn from(err: RenderError) -> Self {
        GenerationError::Rendering(err)
    }
}

impl From<FromUtf8Error> for GenerationError {
    fn from(err: FromUtf8Error) -> Self {
        GenerationError::Utf8(err)
    }
}

impl From<rss::Error> for GenerationError {
    fn from(err: rss::Error) -> Self {
        GenerationError::RSS(err)
    }
}

impl<T: Into<GenerationError>> From<T> for SingleFileError {
    fn from(err: T) -> Self {
        Self::GenerationError(err.into())
    }
}

impl From<InstantiationError> for SingleFileError {
    fn from(err: InstantiationError) -> Self {
        Self::Instantiation(err)
    }
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

pub trait Builder: Sized {
    fn new(output_folder_path: &str) -> Result<Self, TemplateFileError>;
    fn templates(&self) -> &Handlebars;
    fn output_path(&self) -> &str;
    fn base_url(&self) -> &str;
    fn check_post(&self, post: &Post) -> PostStatus;
    fn check_page(&self, page: &Post) -> PostStatus;

    fn perform_post_check(&self, post: &Post) -> bool {
        let status = self.check_post(post);
        if let PostStatus::Error(_) = status {
            fs::remove_dir_all(self.output_path()).unwrap();
        }
        status.resolve("post", post.id())
    }

    fn perform_page_check(&self, page: &Post) -> bool {
        let status = self.check_page(page);
        if let PostStatus::Error(_) = status {
            fs::remove_dir_all(self.output_path()).unwrap();
        }
        status.resolve("page", page.id())
    }

    fn prepare_folder(output_folder_path: &str,
                      delete_existing: bool) -> Result<(), IOError> {
        if fs::metadata(output_folder_path).is_ok() {
            if !delete_existing {
                panic!("Target folder '{}' is non-empty", output_folder_path);
            }

            println!("Cleared previous result");
            fs::remove_dir_all(output_folder_path)?;
        }
        fs::create_dir(output_folder_path)?;
        Ok(())
    }

    fn generate_single_file(&self, filename_in: &str, filename_out: &str) -> Result<(), SingleFileError> {
        let post = Post::load(&PathBuf::from(filename_in), PostIndex::default())?;

        let mut context = context::RenderContext::default();
        context.set_target(&post);

        let mut file = File::create(&filename_out)?;
        let layout = context::LayoutInfo::new(self.base_url().to_string());
        write!(file, "{}", self.templates().render("post", &context.serialize(&layout)?)?)?;
        Ok(())
    }

    fn generate(&self, website: &Website) -> Result<(), GenerationError> {
        let output_folder_path = self.output_path();
        let templates = self.templates();
        let mut context = context::RenderContext::new(website, output_folder_path);

        let mut layout = context::LayoutInfo::new(self.base_url().to_string());

        // Copy css
        copy_folder("./theme/css", &(output_folder_path.to_string() + "/css"))?;

        for page in website.pages.iter() {
            if !self.perform_page_check(page) {
                continue;
            }

            layout.insert_post_in_header(page);
        }
        layout.insert_header("blog", String::from("Blog"));

        for page in website.pages.iter() {
            if !self.perform_page_check(page) {
                continue;
            }

            context.set_target(&page);
            let mut file = context.create_file(output_folder_path)?;
            write!(file, "{}", templates.render("page", &context.serialize(&layout)?)?)?;
            context.copy_images()?;
        }

        let mut channel = ChannelBuilder::default()
            .title("Blog | Johannes Huwald")
            .link("https://jhuwald.com/blog")
            .description("Stuff I have written that you may read.")
            .managing_editor(String::from("hey@jhuwald.com"))
            .webmaster(String::from("hey@jhuwald.com"))
            // .pubdate()
            // .last_build_date()
            .build()
            .unwrap();


        let mut posts = BinaryHeap::new();
        for proj in website.projects.iter() {
            for post in proj.posts.iter() {
                if !self.perform_post_check(post) {
                    continue;
                }
                context.set_target(&post);
                channel.items.push(post.into());
                let mut file = context.create_file(output_folder_path)?;
                let ser = context.serialize(&layout)?;
                write!(file, "{}", templates.render("post", &ser)?)?;
                context.copy_images()?;
                posts.push(ser);
            }
        }

        let file = File::create(&(output_folder_path.to_string() + "/blog/feed"))?;
        channel.write_to(file)?;

        let mut p = Vec::new();
        while let Some(post) = posts.pop() { p.push(post); }

        let index = SerializedBlogIndex { posts: p, layout: &layout };
        write!(index.file(output_folder_path)?, "{}",
               templates.render("project", &index)?)?;
        Ok(())
    }
}

pub struct ReleaseBuilder<'a> {
    templates: Handlebars<'a>,
    output_path: String
}

pub struct PreviewBuilder<'a> {
    templates: Handlebars<'a>,
    output_path: String,
    url: String,
}

fn load_templates<'a>() -> Result<Handlebars<'a>, TemplateFileError> {
    let mut templates = Handlebars::new();
    templates.register_template_file("layout", "./theme/layout.hbs")?;
    templates.register_template_file("page", "./theme/page.hbs")?;
    templates.register_template_file("post", "./theme/post.hbs")?;
    templates.register_template_file("project", "./theme/project.hbs")?;

    templates.register_helper("date", Box::new(render_date));
    Ok(templates)
}

impl Builder for ReleaseBuilder<'_> {
    fn new(output_path: &str) -> Result<Self, TemplateFileError> {
        Ok(ReleaseBuilder {
            output_path: output_path.to_string(),
            templates: load_templates()?
        })
    }

    fn templates(&self) -> &Handlebars {
        &self.templates
    }

    fn output_path(&self) -> &str {
        &self.output_path
    }

    fn base_url(&self) -> &str {
        "https://www.jhuwald.com"
    }

    fn check_post(&self, post: &Post) -> PostStatus {
        if post.published.is_none() {
            return PostStatus::Ignore
        }

        if post.summary().is_none() {
            return PostStatus::Error("non-draft is missing a summary")
        }
        PostStatus::Ok
    }

    fn check_page(&self, page: &Post) -> PostStatus {
        match page.published {
            None => PostStatus::Ignore,
            Some(_) => PostStatus::Ok
        }
    }
}

impl Builder for PreviewBuilder<'_> {
    fn new(output_path: &str) -> Result<Self, TemplateFileError> {
        let url = Path::new(output_path).to_str().unwrap();
        let url = fs::canonicalize(&url).unwrap();
        let url = url.to_str().unwrap().to_string();

        Ok(PreviewBuilder {
            output_path: output_path.to_string(),
            templates: load_templates()?,
            url
        })
    }

    fn templates(&self) -> &Handlebars {
        &self.templates
    }

    fn output_path(&self) -> &str {
        &self.output_path
    }

    fn base_url(&self) -> &str {
        &self.url
    }

    fn check_post(&self, post: &Post) -> PostStatus {
        if post.published.is_some() && post.summary().is_none() {
            return PostStatus::Warning("non-draft is missing a summary")
        }
        PostStatus::Ok
    }

    fn check_page(&self, _page: &Post) -> PostStatus {
        PostStatus::Ok
    }
}

fn copy_folder(folder: &str, target: &str) -> Result<(), IOError> {
    if fs::metadata(target).is_ok() {
        panic!{"copy_folder: target '{}' already exists", target};
    }
    fs::create_dir_all(target)?;

    for entry in fs::read_dir(folder)? {
        let entry = entry?;
        fs::copy(entry.path(), target.to_string() + "/" + entry.file_name().to_str().unwrap())?;
    }

    Ok(())
}

impl From<&Post> for rss::Item {
    fn from(post: &Post) -> Self {
        ItemBuilder::default()
            .title(post.title.clone())
            //.link(post.url)
            //.description(post.summary)
            .author(String::from("Johannes Huwald <hey@jhuwald.com>"))
            //.pubdate(post.published.map_or(None, |d| Some(d.to_rfc2822())))
            //.content(post.content)
            .build().unwrap()
    }
}

struct SerializedBlogIndex<'a> {
    posts: Vec<context::SerializedPost<'a>>,
    layout: &'a context::LayoutInfo
}

impl SerializedBlogIndex<'_> {
    fn file(&self, path: &str) -> Result<File, GenerationError> {
        let filename = String::from(path) + "/blog/index.html";
        Ok(File::create(&filename)?)
    }
}

impl Serialize for SerializedBlogIndex<'_> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer
    {
        let mut s = serializer.serialize_struct("Blog", 5)?;
        s.serialize_field("layout", self.layout)?;
        s.serialize_field("title", "Blog | Johannes Huwald")?;
        s.serialize_field("heading", "Blog")?;
        s.serialize_field("posts", &self.posts)?;

        let css_args: Vec<String> = vec![String::from("../css/style.css")];
        s.serialize_field("css", &css_args)?;
        s.end()
    }
}
