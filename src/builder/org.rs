use orgize::{Element, Org, Event};
use orgize::export::{DefaultHtmlHandler, HtmlHandler};
use std::collections::HashMap;
use std::io::{Error as IOError, Write};
use std::string::FromUtf8Error;
use std::path::Path;
use std::fs;
use chrono::naive;

use super::router::Router;

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
struct OrgHTMLHandler(DefaultHtmlHandler);

impl HtmlHandler<OrgLoadError> for OrgHTMLHandler {
    fn start<W: Write>(&mut self, w: W, element: &Element) -> Result<(), OrgLoadError> {
        match element {
            _ => self.0.start(w, element)?
        }
        Ok(())
    }

    fn end<W: Write>(&mut self, w: W, element: &Element) -> Result<(), OrgLoadError> {
        match element {
            _ => self.0.end(w, element)?
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

    pub fn to_html<T>(&self, router: &T) -> Result<String, OrgLoadError> where T: Router {
        let parser = Org::parse(&self.contents);
        let mut writer = Vec::new();
        let mut handler = OrgHTMLHandler::default();
        parser.write_html_custom(&mut writer, &mut handler)?;
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
