use handlebars::{Handlebars, RenderContext, Helper, Context, JsonRender, HelperResult, Output, RenderError, TemplateFileError};

use std::path::Path;
use std::io::{Error as IOError, Write};
use std::fs;
use std::fs::File;

use serde::ser::{Serialize, SerializeStruct, Serializer};


pub mod website;
pub mod org;
mod router;

use website::{WebsiteLoadError, Website, Post};
use org::{OrgFile, OrgLoadError};
use router::{Router, SingleBlogFolderRouter, NoopRouter};

fn render_date (h: &Helper, _: &Handlebars, _: &Context, _rc: &mut RenderContext, out: &mut dyn Output) -> HelperResult {
    let param = h.param(0).unwrap();

    out.write(param.value().render().as_ref())?;
    Ok(())
}

#[derive(Debug)]
pub enum InstantiationError {
    TemplateNotFound(TemplateFileError)
}

impl From<TemplateFileError> for InstantiationError {
    fn from(err: TemplateFileError) -> Self {
        InstantiationError::TemplateNotFound(err)
    }
}

#[derive(Debug)]
pub enum GenerationError {
    IO(IOError),
    Rendering(RenderError),
    File(WebsiteLoadError)
}

impl From<WebsiteLoadError> for GenerationError {
    fn from(err: WebsiteLoadError) -> Self {
        GenerationError::File(err)
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
        println!("{:?}", entry);
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

    pub fn generate_single_file(filename_in: &str, filename_out: &str) -> Result<(), OrgLoadError> {
        let orgfile = OrgFile::load(Path::new(filename_in))?;
        let router = NoopRouter {  };

        let mut file = File::create(&filename_out)?;
        write!(file, "{}", orgfile.to_html(&router)?)?;

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
        let router = SingleBlogFolderRouter{website: &website};

        // Copy css
        copy_folder("./theme/css", &(output_folder_path.to_string() + "/css"))?;

        for page in website.pages.iter() {
            let ser = PostSerializer { post: &page, router: &router };
            let mut file = ser.prepare_file(output_folder_path)?;
            write!(file, "{}", self.templates.render("page", &ser)?)?;
        }

        for proj in website.projects.iter() {
            //println!("{}", proj.url(&website));
            for post in proj.posts.iter() {
                let ser = PostSerializer { post: &post, router: &router };
                let mut file = ser.prepare_file(output_folder_path)?;
                write!(file, "{}", self.templates.render("post", &ser)?)?;
            }
        }
        let index = SingleBlogViewSerializer { router: &router };
        write!(index.file(output_folder_path)?, "{}",
               self.templates.render("project", &index)?)?;
        Ok(())
    }
}

struct SingleBlogViewSerializer<'a> {
    router: &'a SingleBlogFolderRouter<'a>
}

impl SingleBlogViewSerializer<'_> {
    fn file(&self, path: &str) -> Result<File, GenerationError> {
        let filename = String::from(path) + "/blog/index.html";
        Ok(File::create(&filename)?)
    }
}

impl<'a> Serialize for SingleBlogViewSerializer<'a> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer
    {
        let mut s = serializer.serialize_struct("Blog", 4)?;
        s.serialize_field("title", "Blog | Johannes Huwald")?;
        s.serialize_field("heading", "Blog")?;

        let mut posts: Vec<PostSerializer<'a, SingleBlogFolderRouter>> = vec![];
        for project in &self.router.website.projects {
            for post in &project.posts {
                posts.push(PostSerializer { post, router: self.router });
            }
        }
        s.serialize_field("posts", &posts)?;

        let css_args: Vec<String> = vec![String::from("../css/style.css")];
        s.serialize_field("css", &css_args)?;
        s.end()
    }
}

struct PostSerializer<'a, T> where T: Router {
    post: &'a Post,
    router: &'a T
}

impl<T> PostSerializer<'_, T> where T: Router {
    fn resolve_css_path(&self, css: &str) -> String {
        self.router.css_path_for_post(self.post, css)
    }

    fn prepare_file(&self, path: &str) -> Result<File, GenerationError> {
        let filename = self.router.post_url(self.post, path.to_string());
        println!("Rendering '{}'", filename);
        if fs::metadata(&filename).is_ok() {
            panic!("Duplicate post '{}'", filename)
        }
        fs::create_dir_all(&filename)?;
        let filename = filename + "/index.html";
        Ok(File::create(&filename)?)
    }
}

impl<T> Serialize for PostSerializer<'_, T> where T: Router {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer
    {
        let mut count = 4;
        if let Some(_) = self.post.published { count += 1; }
        if let Some(_) = self.post.last_edit { count += 1; }

        let mut s = serializer.serialize_struct("Post", count)?;
        s.serialize_field("content", &self.post.content(self.router).unwrap())?;
        s.serialize_field("title", &(self.post.title.clone() + " | Johannes Huwald"))?;
        s.serialize_field("heading", &self.post.title)?;
        if let Some(published) = &self.post.published {
            s.serialize_field("published", &published)?;
        }

        if let Some(last_edit) = &self.post.last_edit {
            s.serialize_field("last-edit", &last_edit)?;
        }

        let mut css_args: Vec<String> = vec![self.resolve_css_path("style.css")];
        for css in self.post.extra_css.iter() {
            css_args.push(self.resolve_css_path(&css))
        }
        s.serialize_field("css", &css_args)?;
        s.end()
    }
}
