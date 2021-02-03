use handlebars::{Handlebars, RenderContext, Helper, Context, JsonRender, HelperResult, Output, RenderError, TemplateFileError};

use std::path::Path;
use std::io::{Error as IOError, Write};
use std::fs;
use std::fs::File;

use std::collections::BTreeMap;

pub mod website;
pub mod org;
mod router;

use website::{WebsiteLoadError, Website, Post};

use router::{Router, SingleBlogFolderRouter};

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

        for page in website.pages.iter() {
            let filename = output_folder_path.to_string();
            let filename = router.post_url(&page, filename);
            println!("Rendering '{}'", filename);
            if fs::metadata(&filename).is_ok() {
                panic!("Duplicate post '{}'", filename)
            }
            fs::create_dir_all(&filename)?;
            let filename = filename + "/index.html";
            let mut file = File::create(&filename)?;
            let data = page.serialize();
            write!(file, "{}", self.templates.render("page", &data)?)?;
        }

        for proj in website.projects.iter() {
            //println!("{}", proj.url(&website));
            for post in proj.posts.iter() {
                let filename = output_folder_path.to_string();
                let filename = router.post_url(post, filename);
                println!("Rendering '{}'", filename);
                if fs::metadata(&filename).is_ok() {
                    panic!("Duplicate post '{}'", filename)
                }
                fs::create_dir_all(&filename)?;
                let filename = filename + "/index.html";
                let mut file = File::create(&filename)?;
                let data = post.serialize();
                write!(file, "{}", self.templates.render("post", &data)?)?;
            }
        }
        Ok(())
    }
}

impl Post {
    fn serialize(&self) -> BTreeMap<&str, &String> {
        let mut data = BTreeMap::new();
        data.insert("content", &self.content);
        data.insert("title", &self.title);
        if let Some(published) = &self.published {
            data.insert("published", &published);
        }
        data
    }
}
