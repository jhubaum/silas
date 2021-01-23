use std::convert::From;
use std::env::args;
use std::fs;
use std::fs::File;
use orgize::{Element, Org};
use orgize::export::{DefaultHtmlHandler, HtmlHandler};

use std::io::{Error as IOError, Write};
use std::result::Result;
use std::string::FromUtf8Error;
use handlebars::Handlebars;
use std::collections::BTreeMap;

#[derive(Debug)]
enum ExportError {
    IO(IOError),
    Utf8(FromUtf8Error),
    Template(handlebars::TemplateError)
}

impl From<IOError> for ExportError {
    fn from(err: IOError) -> Self {
        ExportError::IO(err)
    }
}

impl From<FromUtf8Error> for ExportError {
    fn from(err: FromUtf8Error) -> Self {
        ExportError::Utf8(err)
    }
}

impl From<handlebars::TemplateError> for ExportError {
    fn from(err: handlebars::TemplateError) -> Self {
        ExportError::Template(err)
    }
}


#[derive(Default)]
struct CustomHtmlHandler(DefaultHtmlHandler);

impl HtmlHandler<ExportError> for CustomHtmlHandler {
    fn start<W: Write>(&mut self, w: W, element: &Element) -> Result<(), ExportError> {
        match element {
            Element::Link(link) => {
            },
            _ => self.0.start(w, element)?
        }
        Ok(())
    }

    fn end<W: Write>(&mut self, w: W, element: &Element) -> Result<(), ExportError> {
        match element {
            _ => self.0.end(w, element)?
        }
        Ok(())
    }
}

fn create_document() -> Result<String, ExportError>{
    let args: Vec<String> = args().collect();
    let contents = String::from_utf8(fs::read(&args[1])?)?;

    let parser = Org::parse(&contents);
    for event in parser.iter() {
        println!("{:?}", event);
    }
    let mut writer = Vec::new();
    let mut handler = CustomHtmlHandler::default();
    parser.write_html_custom(&mut writer, &mut handler)?;
    Ok(String::from_utf8(writer)?)
}

fn main() -> Result<(), ExportError> {
    let mut reg = Handlebars::new();
    reg.register_template_string("post", "<html>{{{content}}}</html>")?;

    let mut data = BTreeMap::new();
    data.insert("content".to_string(), create_document()?);

    if let Ok(out) = reg.render("post", &data) {
        let mut file = File::create("output.html")?;
        write!(file, "{}", out);
    } else {
        panic!("Handlebars rendering failed")
    }
    Ok(())
}
