use std::path::{Path, PathBuf, Iter};
use std::io::{self, Error as IOError};
use std::fs;

use chrono::naive;


use super::GenerationError;
use super::org;
use super::org::{OrgLoadError, OrgFile, OrgHTMLHandler};
use super::context::RenderContext;

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
    pub projects: Vec<Project>,
    pub path: PathBuf,
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub struct ProjectIndex {
    index: usize
}

pub struct Project {
    pub index: ProjectIndex,
    pub posts: Vec<Post>,
    pub id: String,
    path: PathBuf
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub struct PostIndex {
    pub index: usize,
    pub project: Option<ProjectIndex>
}

impl PostIndex {
    fn without_project(index: usize) -> Self {
        PostIndex { index, project: None }
    }
}

impl Default for PostIndex {
    fn default() -> Self {
        PostIndex { index: 0, project: None }
    }
}

pub struct Post {
    pub index: PostIndex,
    pub title: String,
    pub published: Option<naive::NaiveDate>,
    pub last_edit: Option<naive::NaiveDate>,
    pub extra_css: Vec<String>,
    pub path: PathBuf,
    orgfile: OrgFile
}

impl Website {
    pub fn load(path: &Path) -> Result<Self, WebsiteLoadError> {
        let mut website = Website {
            pages: vec![],
            projects: vec![],
            path: path.to_path_buf()
        };
        for entry in path.read_dir().expect("Path does not exist") {
            if let Ok(entry) = entry {
                let p = entry.path();
                if p.is_dir() {
                    website.projects.push(Project::load(&p, ProjectIndex { index: website.projects.len() })?);
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

    pub fn find_post_from_path(&self, path: PathBuf) -> Option<&Post> {
        let mut iter = path.iter();

        for c in self.path.iter() {
            let cur = iter.next();
            if cur.is_none() || cur.unwrap() != c {
                // given path isn't a subpath of website
                println!("Link points to file outside of website directory");
                return None;
            }
        }

        let proj_name = iter.next();
        if proj_name.is_none() {
            return None;
        }
        let proj_name = proj_name.unwrap().to_str().unwrap().to_string();

        for p in &self.projects {
            if p.id == proj_name {
                return p.find_post_from_path(iter);
            }
        }

        let mut path = self.path.clone();
        path.push(proj_name);

        for p in &self.pages {
            if p.path == path {
                return Some(p);
            }
        }
        return None;
    }

    pub fn get_relative_url(&self, from: &Post, to: &Post) -> String {
        let base = String::from("../");
        if from.index.project == to.index.project {
            return base + to.id();
        }

        match from.index.project {
            None => {
                // to has to have a project
                base + "blog/" + to.id()
            },
            Some(_) => {
                // different or no project
                if to.index.project.is_none() {
                    base + "../" + to.id()
                } else {
                    base + to.id()
                }
            }
        }
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
        Ok(Project { posts, index, path: path.clone(),
                     id: path.file_name().unwrap()
                         .to_str().unwrap().to_string() })
    }

    fn find_post_from_path(&self, iter: Iter) -> Option<&Post> {
        let mut path = self.path.clone();
        for c in iter {
            path.push(c);
        }

        for post in &self.posts {
            if post.path == path {
                return Some(post);
            }
        }
        return None;
    }
}

impl Post {
    pub fn load(filename: &PathBuf, index: PostIndex) -> Result<Self, OrgLoadError> {
        let f = OrgFile::load(filename)?;

        let published = match f.preamble.get("published") {
            None => None,
            Some(d) => Some(org::parse_date(d)?)
        };

        let last_edit = match f.preamble.get("last_edit") {
            None => None,
            Some(d) => Some(org::parse_date(d)?)
        };


        let title = match f.preamble.get("title") {
            None => String::from("NO TITLE"),
            Some(t) => t.clone()
        };

        Ok(Post {
            index, title, published, last_edit,
            path: filename.to_path_buf(),
            orgfile: f,
            extra_css: vec![]
        })
    }

    pub fn id(&self) -> &str {
        &self.orgfile.filename
    }

    pub fn content(&self, context: &RenderContext) -> Result<String, GenerationError> {
        let mut handler = OrgHTMLHandler::new(context);
        self.orgfile.to_html(&mut handler)
    }
}
