use std::path::{Path, PathBuf};
use std::io::{self, Error as IOError};
use std::fs;

use super::org;
use super::org::{OrgLoadError, OrgFile};

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
    pub published: Option<String>,
    pub last_edit: Option<String>,
    pub extra_css: Vec<String>
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
        let f = OrgFile::load(filename)?;

        let published = match f.preamble.get("published") {
            None => None,
            Some(d) => Some(org::parse_date(d)?
                .format("%A, %-d %B, %Y").to_string())
        };

        let last_edit = match f.preamble.get("last_edit") {
            None => None,
            Some(d) => Some(org::parse_date(d)?
                .format("%A, %-d %B, %Y").to_string())
        };


        let title = match f.preamble.get("title") {
            None => String::from("NO TITLE"),
            Some(t) => t.clone()
        };

        Ok(Post {
            index, title, published, last_edit,
            id: f.filename,
            content: f.html,
            extra_css: vec![]
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
