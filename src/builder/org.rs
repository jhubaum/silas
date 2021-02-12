use orgize::{Element, Org, Event};
use orgize::export::{DefaultHtmlHandler, HtmlHandler, HtmlEscape};
use std::collections::HashMap;
use std::io::{Error as IOError, Write};
use std::string::FromUtf8Error;
use std::path::Path;
use std::fs;
use chrono::naive;

use super::context::RenderContext;

#[derive(Debug)]
pub enum OrgLoadError {
    NoFile,
    InvalidFileExtension,
    IO(IOError),
    Utf8(FromUtf8Error),
    Date(chrono::ParseError)
}

impl From<IOError> for OrgLoadError {
    fn from(err: IOError) -> Self {
        OrgLoadError::IO(err)
    }
}

impl From<FromUtf8Error> for OrgLoadError {
    fn from(err: FromUtf8Error) -> Self {
        OrgLoadError::Utf8(err)
    }
}

impl From<chrono::ParseError> for OrgLoadError {
    fn from(err: chrono::ParseError) -> Self {
        OrgLoadError::Date(err)
    }
}

#[derive(Default)]
pub struct OrgHTMLHandler<'a> {
    fallback: DefaultHtmlHandler,
    context: Option<&'a RenderContext<'a>>
}

impl<'a> OrgHTMLHandler<'a> {
    pub fn new(context: &'a RenderContext) -> Self {
        OrgHTMLHandler { context: Some(context),
                         fallback: DefaultHtmlHandler::default() }
    }
}

impl HtmlHandler<OrgLoadError> for OrgHTMLHandler<'_> {
    fn start<W: Write>(&mut self, mut w: W, element: &Element) -> Result<(), OrgLoadError> {
        match element {
            Element::Link(link) => write!(
                w,
                "<a href=\"{}\">{}</a>",
                HtmlEscape(&link.path), // resolve link here
                HtmlEscape(link.desc.as_ref().unwrap_or(&link.path)),
            )?,
            _ => self.fallback.start(w, element)?
        }
        Ok(())
    }

    fn end<W: Write>(&mut self, w: W, element: &Element) -> Result<(), OrgLoadError> {
        match element {
            _ => self.fallback.end(w, element)?
        }
        Ok(())
    }
}

#[derive(Debug)]
pub struct OrgFile {
    pub filename: String,
    pub contents: String,
    pub preamble: HashMap<String, String>,
}

impl OrgFile {
    fn extract_preamble(org: &Org) -> HashMap<String, String>{
        let mut iter = org.iter();
        iter.next(); // Start document
        iter.next(); // Start section

        let mut preamble = HashMap::new();
        loop {
            match iter.next() {
                None => break,
                Some(Event::End(_)) => continue,
                Some(Event::Start(Element::Keyword(k))) => {
                    preamble.insert(
                        k.key.to_string().to_lowercase(),
                        k.value.to_string());
                },
                Some(Event::Start(_)) => break
            };
        }
        preamble
    }

    pub fn to_html<T>(&self, handler: &mut T) -> Result<String, OrgLoadError> where T: HtmlHandler<OrgLoadError> {
        let parser = Org::parse(&self.contents);
        let mut writer = Vec::new();
        parser.write_html_custom(&mut writer, handler)?;
        Ok(String::from_utf8(writer)?)
    }

    pub fn load(filename: &Path) -> Result<Self, OrgLoadError> {
        match filename.extension() {
            None => return Err(OrgLoadError::NoFile),
            Some(ext) => if ext != "org" {
                return Err(OrgLoadError::InvalidFileExtension)
            }
        };

        let contents = String::from_utf8(fs::read(filename)?)?;
        let parser = Org::parse(&contents);

        let filename = filename.file_stem().unwrap()
                               .to_str().unwrap().to_string();

        let preamble = OrgFile::extract_preamble(&parser);
        Ok(OrgFile { filename, preamble, contents })
    }
}

pub fn parse_date(date_str: &str) -> chrono::ParseResult<naive::NaiveDate> {
    naive::NaiveDate::parse_from_str(date_str, "<%Y-%m-%d>")
}
