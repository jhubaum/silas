use std::collections::HashMap;
use std::io::{Error as IOError, Write};
use std::string::FromUtf8Error;

use lazy_static::lazy_static;
use regex::Regex;

use super::website;
use super::website::BlogElement;
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
pub struct Attributes {
    pub caption: Option<String>,
    style: HashMap<String, String>,
    /// A flag to ignore attributes. This is used to ignore attributes in the preamble
    ignore_insert: bool,
}

impl Attributes {
    pub fn insert(&mut self, attribute: &orgize::elements::Keyword) -> Result<bool, String> {
        if self.ignore_insert {
            return Ok(true);
        }

        match attribute.key.as_ref() {
            "ATTR_HTML" => {
                lazy_static! {
                    static ref CSS_RE: Regex =
                        Regex::new(":style (?P<attr>[[:alpha:]]+): (?P<value>[[:word:]%]+);?$")
                            .unwrap();
                }
                if CSS_RE
                    .captures(&attribute.value)
                    .and_then(|cap| {
                        self.style.insert(
                            cap.name("attr").unwrap().as_str().to_string(),
                            cap.name("value").unwrap().as_str().to_string(),
                        );
                        Some(())
                    })
                    .is_none()
                {
                    return Err(format!(
                        "Unable to handle HTML attribute `{}`",
                        attribute.value
                    ));
                }
            }
            "CAPTION" => self.caption = Some(attribute.value.to_string()),
            _ => return Ok(false),
        }
        Ok(true)
    }

    pub fn get_inline_style(&self) -> String {
        if self.style.len() == 0 {
            String::from("")
        } else {
            // this can probably be implemented more efficiently. But it works for now. So I don't care.
            format!(
                " style=\"{}\"",
                self.style
                    .iter()
                    .map(|tup| format!("{}: {};", tup.0, tup.1))
                    .collect::<Vec<String>>()
                    .join(" ")
            )
        }
    }

    /// Create an attribute instance that ignores all inputs
    pub fn none() -> Self {
        let mut tmp = Self::default();
        tmp.ignore_insert = true;
        tmp
    }
}

#[derive(Default)]
pub struct OrgHTMLHandler<'a> {
    website: Option<&'a website::Website>,
    post: Option<&'a website::OrgFile>,
    fallback: DefaultHtmlHandler,
    attributes: Attributes,
    base_url: String,
    image_deps: Vec<String>,
    first_footnote: bool
}

pub struct RenderResult {
    pub content: String,
    pub image_deps: Vec<String>,
}

impl<'a> OrgHTMLHandler<'a> {
    pub fn render_post<T: Mode>(
        website: &website::Website,
        post: &website::OrgFile,
        mode: &T,
    ) -> Result<RenderResult, HTMLExportError> {
        let parser = Org::parse(&post.contents);
        let mut handler = OrgHTMLHandler {
            website: Some(website),
            post: Some(post),
            fallback: DefaultHtmlHandler::default(),
            attributes: Attributes::none(),
            base_url: mode.base_url(),
            image_deps: Vec::new(),
            first_footnote: true,
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
                    let alt = link.desc.as_ref().map(|s| s.as_ref());
                    match &self.attributes.caption {
                        None => {
                            self.render_image(w, &target, alt, &self.attributes.get_inline_style())?
                        }
                        Some(c) => self.render_figure(w, &target, alt, &c)?,
                    };
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
                let res = website.resolve_path(&path);
                assert!(
                    res.is_some(),
                    "Unable to resolve link `{}` in {:?}",
                    link,
                    post.path
                );
                let link = res.unwrap();
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

    fn render_figure<W: Write>(
        &self,
        w: &mut W,
        src: &str,
        alt: Option<&str>,
        caption: &str,
    ) -> Result<(), HTMLExportError> {
        write!(w, "<figure{}>", self.attributes.get_inline_style())?;
        self.render_image(w, src, alt, " style=\"width: 100%\"")?;
        write!(w, "<figcaption>{}</figcaption></figure>", caption)?;

        Ok(())
    }

    fn render_image<W: Write>(
        &self,
        w: &mut W,
        src: &str,
        alt: Option<&str>,
        style: &str,
    ) -> Result<(), HTMLExportError> {
        if let Some(desc) = alt {
            write!(
                w,
                "<img src=\"./{}\" alt=\"{}\"{}>",
                HtmlEscape(src),
                HtmlEscape(&desc),
                style
            )?;
        } else {
            write!(w, "<img src=\"./{}\"{}>", HtmlEscape(src), style)?;
        }
        Ok(())
    }
}

impl HtmlHandler<HTMLExportError> for OrgHTMLHandler<'_> {
    fn start<W: Write>(&mut self, mut w: W, element: &Element) -> Result<(), HTMLExportError> {
        match element {
            Element::Keyword(keyword) => self.attributes.insert(keyword).map_or_else(
                |err| {
                    panic!("{:?}: {}", self.post.unwrap().path, err);
                },
                |handled| {
                    if !handled {
                        println!(
                            "Warning: Unhandled attribute `{}` in {:?}",
                            keyword.key,
                            self.post.unwrap().path
                        )
                    }
                },
            ),
            Element::Link(link) => {
                if self.write_link(&mut w, &link)? {
                    self.fallback.start(w, element)?;
                }
            },
            Element::FnRef(fnref) => {
                write!(w, "<sup id=\"ref{0}\"><a href=\"#fn{0}\">{0}</a></sup>", fnref.label)?;
            },
            Element::FnDef(fndef) => {
                if self.first_footnote {
                    self.first_footnote = false;
                    write!(w, "<p>---</p>")?;
                }
                write!(w, "<div id=\"fn{0}\" class=\"fndef\"><a href=\"#ref{0}\">{0}</a>: ", fndef.label)?;
            },
            Element::Document { .. } => {}
            _ => self.fallback.start(w, element)?,
        };

        Ok(())
    }

    fn end<W: Write>(&mut self, mut w: W, element: &Element) -> Result<(), HTMLExportError> {
        match element {
            Element::FnDef(_) => {
                write!(w, "</div>")?
            },
            Element::Document { .. } => {}
            _ => self.fallback.end(w, element)?,
        }

        // Reset attributes after each element with content
        match element {
            // don't reset it on these elements
            Element::Keyword(_) => {}
            _ => self.attributes = Attributes::default(),
        };

        Ok(())
    }
}

#[derive(Default)]
pub struct OrgExtractGenerator {
    written_length: usize,
    finished_writing: bool,
}

impl OrgExtractGenerator {
    const MAX_LENGTH: usize = 200;

    pub fn generate(file: &website::OrgFile) -> Result<String, HTMLExportError> {
        let parser = Org::parse(&file.contents);
        let mut handler = OrgExtractGenerator::default();
        let mut writer = Vec::new();
        parser.write_html_custom(&mut writer, &mut handler)?;
        if handler.finished_writing {
            write!(writer, "...")?;
        }

        Ok(String::from_utf8(writer)?)
    }

    fn write<W: Write>(&mut self, mut w: W, text: &str) -> Result<(), HTMLExportError> {
        if self.finished_writing {
            return Ok(());
        }
        if self.written_length + text.len() <= Self::MAX_LENGTH {
            write!(w, "{}", text)?;
            return Ok(());
        }
        // found last element to be written. Split it at a fitting word
        let index = text.find(char::is_whitespace);
        if index.is_none() {
            // text is only one word, ignore it completely
            self.finished_writing = true;
            return Ok(());
        }
        let mut index = index.unwrap();
        loop {
            let next = text[index..].find(char::is_whitespace);
            match next {
                None => {
                    break;
                }
                Some(d) => {
                    if self.written_length + index + d > Self::MAX_LENGTH {
                        break;
                    }
                    index += d + 1;
                }
            };
        }

        self.finished_writing = true;
        write!(w, "{}", &text[..index - 1])?;
        Ok(())
    }
}

impl HtmlHandler<HTMLExportError> for OrgExtractGenerator {
    fn start<W: Write>(&mut self, w: W, element: &Element) -> Result<(), HTMLExportError> {
        match element {
            Element::Link(link) => {
                if let Some(desc) = &link.desc {
                    self.write(w, desc)?;
                }
            }
            Element::Text { value } => {
                self.write(w, value)?;
            }
            _ => {}
        }
        Ok(())
    }

    fn end<W: Write>(&mut self, _w: W, _element: &Element) -> Result<(), HTMLExportError> {
        Ok(())
    }
}

impl website::OrgFile {
    pub fn render_html<T: Mode>(
        &self,
        website: &website::Website,
        mode: &T,
    ) -> Result<RenderResult, HTMLExportError> {
        OrgHTMLHandler::render_post(website, self, mode)
    }
}
