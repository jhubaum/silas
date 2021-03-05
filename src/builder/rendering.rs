use std::io::{Error as IOError, Write};
use std::string::FromUtf8Error;

use super::website_new;
use super::website_new::BlogElement;
use super::Mode;

use orgize::elements;
use orgize::export::{DefaultHtmlHandler, HtmlEscape, HtmlHandler};
use orgize::{Element, Org};

#[derive(Debug)]
pub enum HTMLExportError {
    UTF8(FromUtf8Error),
    IO(IOError),
}

impl From<FromUtf8Error> for HTMLExportError {
    fn from(err: FromUtf8Error) -> Self {
        Self::UTF8(err)
    }
}

impl From<IOError> for HTMLExportError {
    fn from(err: IOError) -> Self {
        Self::IO(err)
    }
}

enum ResolvedInternalLink {
    Post(String),
    Image(String),
}

#[derive(Default)]
pub struct OrgHTMLHandler<'a> {
    website: Option<&'a website_new::Website>,
    post: Option<&'a website_new::OrgFile>,
    fallback: DefaultHtmlHandler,
    attributes: Vec<String>,
    base_url: String,
    image_deps: Vec<String>,
}

pub struct RenderResult {
    pub content: String,
    pub image_deps: Vec<String>,
}

impl<'a> OrgHTMLHandler<'a> {
    pub fn render_post<T: Mode>(
        website: &website_new::Website,
        post: &website_new::OrgFile,
        mode: &T,
    ) -> Result<RenderResult, HTMLExportError> {
        let parser = Org::parse(&post.contents);
        let mut handler = OrgHTMLHandler {
            website: Some(website),
            post: Some(post),
            fallback: DefaultHtmlHandler::default(),
            attributes: Vec::new(),
            base_url: mode.base_url(),
            image_deps: Vec::new(),
        };
        let mut writer = Vec::new();
        parser.write_html_custom(&mut writer, &mut handler)?;
        Ok(RenderResult {
            content: String::from_utf8(writer)?,
            image_deps: handler.image_deps,
        })
    }

    /// return true if the fallback rendering should be used
    fn write_link<W: Write>(
        &mut self,
        w: &mut W,
        link: &elements::Link,
    ) -> Result<bool, HTMLExportError> {
        let mut link_it = link.path.split(":");
        let link_type = link_it.next();
        let link_path = link_it.next().unwrap();

        if link_it.next().is_some() {
            println!("Found link to a section ({}), which is not supported yet. The link will simply point to the file", link.path);
        }

        if link_type.is_none() {
            println!(
                "Warning: Link {} in {:?} has no type. It will not be resolved",
                link.path,
                self.post.unwrap().path
            );
            return Ok(true);
        }

        match link_type.unwrap() {
            // external links don't need to be resolved
            "https" | "http" | "mailto" => return Ok(true),
            "file" => match self.resolve_link(&link_path)? {
                ResolvedInternalLink::Post(target) => {
                    write!(
                        w,
                        "<a href=\"{}\">{}</a>",
                        HtmlEscape(&target),
                        HtmlEscape(link.desc.as_ref().map_or(target.as_str(), |s| &s))
                    )?;
                }
                ResolvedInternalLink::Image(target) => {
                    let desc = match &link.desc {
                        None => None,
                        Some(s) => Some(s.as_ref()),
                    };
                    self.render_image(w, &target, desc)?;
                }
            },
            lt => {
                println!(
                    "Warning: Unknown link type {} in file {:?}. Link will not be resolved",
                    lt,
                    self.post.unwrap().path
                );
                return Ok(true);
            }
        };
        Ok(false)
    }

    fn resolve_link(&mut self, link: &str) -> Result<ResolvedInternalLink, HTMLExportError> {
        let website = self.website.unwrap();
        let post = self.post.unwrap();

        match link.split(".").last().unwrap() {
            "org" => {
                let path = post.resolve_link(link);
                let link = website.resolve_path(&path).unwrap();
                Ok(ResolvedInternalLink::Post(
                    link.url(&website, self.base_url.clone()),
                ))
            }
            "png" | "jpeg" => {
                self.image_deps.push(String::from(link));
                Ok(ResolvedInternalLink::Image(link.to_string()))
            }
            _ => panic!("Unknown file ending for link in {:?}", post.path),
        }
    }

    fn render_image<W: Write>(
        &self,
        w: &mut W,
        src: &str,
        alt: Option<&str>,
    ) -> Result<(), HTMLExportError> {
        let mut css = Vec::new();
        for attr in &self.attributes {
            if &attr[0..7] != ":style " {
                panic!(
                    "Unable to handle attribute {} for rendering image",
                    self.attributes[0]
                );
            }
            css.push(&attr[7..attr.len()]);
        }
        let css = if css.len() == 0 {
            String::from(" ")
        } else {
            format!(" style=\"{}\"", css.join(" "))
        };

        if let Some(desc) = alt {
            write!(
                w,
                "<img src=\"./{}\" alt=\"{}\"{}>",
                HtmlEscape(src),
                HtmlEscape(&desc),
                css
            )?;
        } else {
            write!(w, "<img src=\"./{}\"{}>", HtmlEscape(src), css)?;
        }
        Ok(())
    }

    fn insert_attribute(&mut self, attribute: &orgize::elements::Keyword) {
        match attribute.key.as_ref() {
            "ATTR_HTML" => self.attributes.push(attribute.value.to_string()),
            _ => {}
        }
    }
}

impl HtmlHandler<HTMLExportError> for OrgHTMLHandler<'_> {
    fn start<W: Write>(&mut self, mut w: W, element: &Element) -> Result<(), HTMLExportError> {
        match element {
            Element::Keyword(keyword) => self.insert_attribute(keyword),
            Element::Link(link) => {
                if self.write_link(&mut w, &link)? {
                    self.fallback.start(w, element)?;
                }
            }
            Element::Document { .. } => {}
            _ => self.fallback.start(w, element)?,
        };

        // Reset attributes after each element with content
        if match element {
            Element::Keyword(_) => false,
            Element::Paragraph { .. } => false,
            _ => true,
        } {
            self.attributes.clear();
        }

        Ok(())
    }

    fn end<W: Write>(&mut self, w: W, element: &Element) -> Result<(), HTMLExportError> {
        match element {
            Element::Document { .. } => {}
            _ => self.fallback.end(w, element)?,
        }
        Ok(())
    }
}

impl website_new::OrgFile {
    pub fn render_html<T: Mode>(
        &self,
        website: &website_new::Website,
        mode: &T,
    ) -> Result<RenderResult, HTMLExportError> {
        OrgHTMLHandler::render_post(website, self, mode)
    }
}
