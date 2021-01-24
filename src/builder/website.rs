use orgize::{Element, Org};
use orgize::export::{DefaultHtmlHandler, HtmlHandler};
use std::string::FromUtf8Error;
use std::io::{Error as IOError, Write};
use std::fs;

#[derive(Debug)]
pub enum OrgLoadError {
    IO(IOError),
    Utf8(FromUtf8Error)
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

#[derive(Default)]
pub struct OrgHTMLHandler(DefaultHtmlHandler);

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

pub struct Post {
    pub content: String,
    pub title: String,
    pub published: String
}

impl Post {
    pub fn load(filename: &str, handler: &mut OrgHTMLHandler) -> Result<Self, OrgLoadError> {
        let contents = String::from_utf8(fs::read(filename)?)?;
        let parser = Org::parse(&contents);
        //for event in parser.iter() {
        //    println!("{:?}", event);
        //}
        let mut writer = Vec::new();
        parser.write_html_custom(&mut writer, handler)?;

        Ok(Post {
            content: String::from_utf8(writer)?,
            title: "This is a test title".to_string(),
            published: "<2021-02-28>".to_string()
        })
    }
}
