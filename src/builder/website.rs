use orgize::{Element, Org};
use orgize::export::{DefaultHtmlHandler, HtmlHandler};
use std::path::{Path, PathBuf};
use std::string::FromUtf8Error;
use std::io::{self, Error as IOError, Write};
use std::fs;

#[derive(Debug)]
pub enum WebsiteLoadError {
    Project(ProjectLoadError),
    Org(OrgLoadError),
}

#[derive(Debug)]
pub enum ProjectLoadError {
    Org(OrgLoadError),
    IO(IOError),
}

#[derive(Debug)]
pub enum OrgLoadError {
    IO(IOError),
    Utf8(FromUtf8Error)
}

impl From<ProjectLoadError> for WebsiteLoadError {
    fn from(err: ProjectLoadError) -> Self {
        WebsiteLoadError::Project(err)
    }
}

impl From<OrgLoadError> for WebsiteLoadError {
    fn from(err: OrgLoadError) -> Self {
        WebsiteLoadError::Org(err)
    }
}

impl From<OrgLoadError> for ProjectLoadError {
    fn from(err: OrgLoadError) -> Self {
        ProjectLoadError::Org(err)
    }
}

impl From<IOError> for ProjectLoadError {
    fn from(err: IOError) -> Self {
        ProjectLoadError::IO(err)
    }
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


pub struct Website {
    pub pages: Vec<Post>,
    pub projects: Vec<Project>
}

#[derive(Copy, Clone)]
pub struct ProjectIndex {
    index: usize
}

pub struct Project {
    pub index: ProjectIndex,
    pub posts: Vec<Post>,
    pub id: String,
}

pub struct PostIndex {
    pub index: usize,
    pub project: Option<ProjectIndex>
}

impl PostIndex {
    fn without_project(index: usize) -> Self {
        PostIndex { index, project: None }
    }
}

pub struct Post {
    pub index: PostIndex,
    pub id: String,
    pub content: String,
    pub title: String,
    pub published: String
}

impl Website {
    pub fn load(path: &Path) -> Result<Self, WebsiteLoadError> {
        let mut website = Website {
            pages: vec![],
            projects: vec![]
        };
        for entry in path.read_dir().expect("Path does not exist") {
            if let Ok(entry) = entry {
                let p = entry.path();
                if p.is_dir() {
                    website.projects.push(Project::load(&p, ProjectIndex { index: website.projects.len()})?);
                } else if p.file_name().unwrap() == "index.org" {
                    // create index file here
                } else if p.extension().unwrap() == "org" {
                    let post = Post::load(&p, PostIndex::without_project(website.pages.len()))?;
                    website.pages.push(post);
                }
            }
        }

        Ok(website)
    }

    pub fn url(&self) -> String {
        String::from("https://jhuwald.com")
    }
}


fn visit_dirs(dir: &Path, files: &mut Vec::<PathBuf>) -> io::Result<()> {
    if dir.is_dir() {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                visit_dirs(&path, files)?;
            } else {
                files.push(path);
            }
        }
    }
    Ok(())
}

impl Project {
    pub fn load(path: &PathBuf, index: ProjectIndex) -> Result<Self, ProjectLoadError> {
        println!("Create project {:?}", path);

        let mut filenames = Vec::<PathBuf>::new();
        visit_dirs(path, &mut filenames)?;
        let mut posts = Vec::<Post>::new();
        for f in filenames.iter() {
            if f.file_name().unwrap() == "index.org" {
                // create index file here
            } else if f.extension().unwrap() == "org" {
                posts.push(Post::load(&f, PostIndex { index: posts.len(), project: Some(index) })?);
            }
        }
        Ok(Project { posts, index, id: path.file_name().unwrap().to_str().unwrap().to_string() })
    }

    pub fn url(&self, website: &Website) -> String {
        website.url() + "/" + &self.id
    }
}

impl Post {
    pub fn load(filename: &PathBuf, index: PostIndex) -> Result<Self, OrgLoadError> {
        println!("Load post {:?}", filename);
        let contents = String::from_utf8(fs::read(filename)?)?;
        let parser = Org::parse(&contents);
        //for event in parser.iter() {
        //    println!("{:?}", event);
        //}
        let mut writer = Vec::new();
        let mut handler = OrgHTMLHandler::default();
        parser.write_html_custom(&mut writer, &mut handler)?;

        Ok(Post {
            index,
            id: filename.file_stem().unwrap().to_str().unwrap().to_string(),
            content: String::from_utf8(writer)?,
            title: "This is a test title".to_string(),
            published: "<2021-02-28>".to_string()
        })
    }

    pub fn url(&self, website: &Website) -> String {
        match self.index.project {
            None => website.url() + "/" + &self.id,
            Some(p) => {
                let proj = &website.projects[p.index];
                website.url() + "/" + &proj.id + "/" + &self.id
            }
        }
    }
}
