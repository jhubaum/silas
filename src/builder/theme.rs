use std::fs;
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
        let target = String::from(output_folder_path) + "/css";

        if fs::metadata(&target).is_ok() {
            panic! {"copy_folder: target '{}' already exists", &target};
        }
        fs::create_dir_all(&target)?;

        let css_path = self.theme_dir.to_string() + "/css";
        for entry in fs::read_dir(css_path)? {
            let entry = entry?;
            let file_name = target.to_string() + "/" + entry.file_name().to_str().unwrap();
            fs::copy(entry.path(), file_name)?;
        }

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
