use std::fs;
use std::path::{Path, PathBuf};
use std::fs::File;
use std::io::{Error as IOError, Write};

use chrono::naive::NaiveDate;
use handlebars::{
    Context, Handlebars, Helper, HelperResult, JsonRender, Output, RenderContext, TemplateFileError,
};
use serde::ser::Serialize;

fn render_date(
    h: &Helper,
    _: &Handlebars,
    _: &Context,
    _rc: &mut RenderContext,
    out: &mut dyn Output,
) -> HelperResult {
    let param = h.param(0).unwrap();

    let date = param.value().render();
    let date = NaiveDate::parse_from_str(date.as_ref(), "%Y-%m-%d").unwrap();
    out.write(&date.format("%A, %d. %B %Y").to_string())?;
    Ok(())
}

fn copy_folder_recursively<U: AsRef<Path>, V: AsRef<Path>+std::fmt::Display>(from: U, to: V) -> Result<(), std::io::Error> {
    if fs::metadata(&to).is_ok() {
        panic! {"copy_folder_recursively: target '{}' already exists", &to};
    }
    let mut stack = Vec::new();
    stack.push(PathBuf::from(from.as_ref()));

    let output_root = PathBuf::from(to.as_ref());
    let input_root = PathBuf::from(from.as_ref()).components().count();

    while let Some(working_path) = stack.pop() {
        // Generate a relative path
        let src: PathBuf = working_path.components().skip(input_root).collect();

        // Create a destination if missing
        let dest = if src.components().count() == 0 {
            output_root.clone()
        } else {
            output_root.join(&src)
        };
        if fs::metadata(&dest).is_err() {
            fs::create_dir_all(&dest)?;
        }

        for entry in fs::read_dir(working_path)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
            } else if let Some(filename) = path.file_name() {
                let dest_path = dest.join(filename);
                fs::copy(&path, &dest_path)?;
            }
        }
    }

    Ok(())
}


#[derive(Debug)]
pub enum ThemeError {
    Template(TemplateFileError),
    IO(IOError),
}

#[derive(Debug)]
pub enum RenderError {
    Template(handlebars::RenderError),
    IO(IOError),
}

impl From<TemplateFileError> for ThemeError {
    fn from(err: TemplateFileError) -> Self {
        Self::Template(err)
    }
}

impl From<IOError> for ThemeError {
    fn from(err: IOError) -> Self {
        Self::IO(err)
    }
}

impl From<handlebars::RenderError> for RenderError {
    fn from(err: handlebars::RenderError) -> Self {
        Self::Template(err)
    }
}

impl From<IOError> for RenderError {
    fn from(err: IOError) -> Self {
        Self::IO(err)
    }
}

pub struct Theme<'a> {
    templates: Handlebars<'a>,
    theme_dir: &'a str,
}

impl<'a> Theme<'a> {
    pub fn load(path: &'a str) -> Result<Self, ThemeError> {
        let mut templates = Handlebars::new();
        for template in ["layout", "page", "post", "project"].iter() {
            let filename = format!("{}/{}.hbs", path, template);
            templates.register_template_file(template, filename)?;
        }

        templates.register_helper("date", Box::new(render_date));

        Ok(Theme {
            templates,
            theme_dir: path,
        })
    }

    pub fn copy_files(&self, output_folder_path: &str) -> Result<(), IOError> {
        copy_folder_recursively(self.theme_dir.to_string() + "/css",
                                String::from(output_folder_path) + "/css")?;

        copy_folder_recursively(self.theme_dir.to_string() + "/js",
                                String::from(output_folder_path) + "/js")?;

        fs::copy(
            self.theme_dir.to_string() + "/favicon.png",
            output_folder_path.to_string() + "/favicon.png",
        )?;
        Ok(())
    }

    pub fn render<TData: Serialize>(
        &self,
        file: &mut File,
        template: &str,
        data: &TData,
    ) -> Result<(), RenderError> {
        write!(file, "{}", self.templates.render(template, data)?)?;
        Ok(())
    }
}
