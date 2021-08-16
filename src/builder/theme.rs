use std::fs;
use std::fs::File;
use std::io::{Error as IOError, Write};

use chrono::naive::NaiveDate;
use handlebars::{
    Context, Handlebars, Helper, HelperResult, JsonRender, Output, RenderContext, TemplateFileError,
};
use serde::ser::Serialize;

use super::fileutil::copy_folder_recursively;
use super::website::ProjectType;

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
    theme_dir: String,
}

pub enum TemplateType {
    Post,
    Project(ProjectType),
    Page,
}

impl TemplateType {
    fn to_template_name(&self) -> &'static str {
        match self {
            Self::Post => "post",
            Self::Project(pt) => match pt {
                ProjectType::Catalogue => "projects/catalogue",
                ProjectType::MultiPart => "projects/multi",
            },
            Self::Page => "page",
        }
    }
}

impl<'a> Theme<'a> {
    pub fn load(path: &str) -> Result<Self, ThemeError> {
        let mut templates = Handlebars::new();
        for template in [
            "layout",
            "page",
            "post",
            "projects/catalogue",
            "projects/multi",
        ]
        .iter()
        {
            let filename = format!("{}/{}.hbs", path, template);
            templates.register_template_file(template, filename)?;
        }

        templates.register_helper("date", Box::new(render_date));

        Ok(Theme {
            templates,
            theme_dir: path.into(),
        })
    }

    pub fn copy_files(&self, output_folder_path: &str) -> Result<(), IOError> {
        copy_folder_recursively(
            self.theme_dir.to_string() + "/css",
            String::from(output_folder_path) + "/css",
        )?;

        copy_folder_recursively(
            self.theme_dir.to_string() + "/js",
            String::from(output_folder_path) + "/js",
        )?;

        fs::copy(
            self.theme_dir.to_string() + "/favicon.png",
            output_folder_path.to_string() + "/favicon.png",
        )?;
        Ok(())
    }

    pub fn render<TData: Serialize>(
        &self,
        file: &mut File,
        template: TemplateType,
        data: &TData,
    ) -> Result<(), RenderError> {
        write!(
            file,
            "{}",
            self.templates.render(template.to_template_name(), data)?
        )?;
        Ok(())
    }
}
