use orgize::{Element, Org, Event};
use orgize::elements;
use orgize::export::{DefaultHtmlHandler, HtmlHandler, HtmlEscape};
use std::collections::HashMap;
use std::io::{Error as IOError, Write};
use std::string::FromUtf8Error;
use std::path::Path;
use std::fs;
use chrono::naive;

use super::GenerationError;
use super::context::{RenderContext, ResolvedInternalLink};

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


#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OrgFile {
    pub filename: String,
    pub contents: String,
    pub preamble: HashMap<String, String>,
}

impl OrgFile {
    fn extract_preamble(org: &Org, filename: &Path) -> HashMap<String, String>{
        let mut iter = org.iter();
        iter.next(); // Start document
        iter.next(); // Start section

        let mut preamble = HashMap::new();
        loop {
            match iter.next() {
                None => break,
                Some(Event::End(_)) => continue,
                Some(Event::Start(Element::Keyword(k))) => {
                    if k.value.len() == 0 {
                        println!("Warning: encountered empty keyword '{}' while parsing org file {:?}", k.key, filename);
                    } else {
                        preamble.insert(
                            k.key.to_string().to_lowercase(),
                            k.value.to_string());
                    }
                },
                Some(Event::Start(_)) => break
            };
        }
        preamble
    }

    pub fn to_html<T>(&self, handler: &mut T) -> Result<String, GenerationError> where T: HtmlHandler<GenerationError> {
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

        /*
        for event in parser.iter() {
            println!("{:?}", event);
        }
        */

        let preamble = OrgFile::extract_preamble(&parser, filename);
        let filename = filename.file_stem().unwrap()
                               .to_str().unwrap().to_string();

        Ok(OrgFile { filename, preamble, contents })
    }
}

pub fn parse_date(date_str: &str) -> chrono::ParseResult<naive::NaiveDate> {
    naive::NaiveDate::parse_from_str(date_str, "<%Y-%m-%d>")
}
