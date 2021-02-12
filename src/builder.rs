use handlebars::{Handlebars, RenderContext, Helper, Context, JsonRender, HelperResult, Output, RenderError, TemplateFileError};

use std::path::{Path, PathBuf};
use std::io::{Error as IOError, Write};
use std::fs;
use std::fs::File;

use serde::ser::{Serialize, Serializer, SerializeStruct};

pub mod website;
pub mod org;
mod context;

use website::{WebsiteLoadError, Website, Post, PostIndex};
use org::OrgLoadError;

fn render_date (h: &Helper, _: &Handlebars, _: &Context, _rc: &mut RenderContext, out: &mut dyn Output) -> HelperResult {
    let param = h.param(0).unwrap();

    out.write(param.value().render().as_ref())?;
    Ok(())
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
    Duplicate
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

impl<T> From<T> for SingleFileError where T: Into<GenerationError> {
    fn from(err: T) -> Self {
        Self::GenerationError(err.into())
    }
}

impl From<InstantiationError> for SingleFileError {
    fn from(err: InstantiationError) -> Self {
        Self::Instantiation(err)
    }
}

pub struct Builder<'a> {
    templates: Handlebars<'a>,
    path: String
}

fn copy_folder(folder: &str, target: &str) -> Result<(), IOError> {
    if fs::metadata(target).is_ok() {
        panic!{"copy_folder: target '{}' already exists", target};
    }
    fs::create_dir_all(target)?;

    for entry in fs::read_dir(folder)? {
        let entry = entry?;
        fs::copy(entry.path(), target.to_string() + "/" + &entry.file_name().to_str().unwrap())?;
    }

    Ok(())
}

impl Builder<'_> {
    pub fn new(blog_path: &str) -> Result<Self, InstantiationError> {
        let mut templates = Handlebars::new();
        templates.register_template_file("layout", "./theme/layout.hbs")?;
        templates.register_template_file("page", "./theme/page.hbs")?;
        templates.register_template_file("post", "./theme/post.hbs")?;
        templates.register_template_file("project", "./theme/project.hbs")?;

        templates.register_helper("date", Box::new(render_date));

        Ok(Builder {
            templates,
            path: String::from(blog_path)
        })
    }

    pub fn generate_single_file(filename_in: &str, filename_out: &str) -> Result<(), SingleFileError> {
        let builder = Builder::new("")?;

        let post = Post::load(&PathBuf::from(filename_in), PostIndex::none())?;

        let mut context = context::RenderContext::default();
        context.set_target(&post);

        let mut file = File::create(&filename_out)?;
        write!(file, "{}", builder.templates.render("post", &context.serialize()?)?)?;
        Ok(())
    }

    pub fn generate(&mut self, output_folder_path: &str, delete_existing: bool) -> Result<(), GenerationError> {
        if fs::metadata(output_folder_path).is_ok() {
            if !delete_existing {
                panic!("Target folder '{}' is non-empty", output_folder_path);
            }

            println!("Cleared previous result");
            fs::remove_dir_all(output_folder_path)?;
        }
        fs::create_dir(output_folder_path)?;

        let website = Website::load(Path::new(&self.path))?;
        let mut context = context::RenderContext::new(&website);

        // Copy css
        copy_folder("./theme/css", &(output_folder_path.to_string() + "/css"))?;

        for page in website.pages.iter() {
            context.set_target(&page);
            let mut file = context.create_file(output_folder_path)?;
            write!(file, "{}", self.templates.render("page", &context.serialize()?)?)?;
        }

        let mut posts = Vec::new();
        for proj in website.projects.iter() {
            for post in proj.posts.iter() {
                context.set_target(&post);
                let mut file = context.create_file(output_folder_path)?;
                let ser = context.serialize()?;
                write!(file, "{}", self.templates.render("post", &ser)?)?;
                posts.push(ser);
            }
        }
        let index = SerializedBlogIndex { posts };
        write!(index.file(output_folder_path)?, "{}",
               self.templates.render("project", &index)?)?;
        Ok(())
    }
}

struct SerializedBlogIndex {
    posts: Vec<context::SerializedPost>,
}

impl SerializedBlogIndex {
    fn file(&self, path: &str) -> Result<File, GenerationError> {
        let filename = String::from(path) + "/blog/index.html";
        Ok(File::create(&filename)?)
    }
}

impl Serialize for SerializedBlogIndex {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer
    {
        let mut s = serializer.serialize_struct("Blog", 4)?;
        s.serialize_field("title", "Blog | Johannes Huwald")?;
        s.serialize_field("heading", "Blog")?;
        s.serialize_field("posts", &self.posts)?;

        let css_args: Vec<String> = vec![String::from("../css/style.css")];
        s.serialize_field("css", &css_args)?;
        s.end()
    }
}
