use std::collections::{HashMap, HashSet};
use std::io::Error as IOError;
use std::path::{Path, PathBuf};
use std::string::FromUtf8Error;

use orgize::{Element, Event, Org};
use std::fs;

#[derive(Debug)]
pub enum LoadError {
    IO(IOError),
    DuplicateFileID(String),
    UTF8(FromUtf8Error),
    Date(chrono::ParseError),
}

impl From<IOError> for LoadError {
    fn from(err: IOError) -> Self {
        Self::IO(err)
    }
}

impl From<FromUtf8Error> for LoadError {
    fn from(err: FromUtf8Error) -> Self {
        Self::UTF8(err)
    }
}

impl From<chrono::ParseError> for LoadError {
    fn from(err: chrono::ParseError) -> Self {
        Self::Date(err)
    }
}

pub struct Website {
    pub projects: HashMap<String, Project>,
    pub pages: HashMap<PathBuf, OrgFile>,
}

pub struct Project {
    pub posts: HashMap<PathBuf, OrgFile>,
}

#[derive(Clone)]
pub struct OrgFile {
    id: String,
    preamble: HashMap<String, String>,
    pub path: PathBuf,
    pub contents: String,
    pub published: Option<chrono::naive::NaiveDate>,
    pub last_edit: Option<chrono::naive::NaiveDate>,
}

pub trait BlogElement {
    fn url(&self, website: &Website, base: String) -> String;
    fn title(&self) -> &str;
    fn description(&self) -> &str;
}

const IGNORED_FOLDERS: [&str; 1] = ["drafts"];
const IGNORED_FILES: [&str; 2] = ["ideas.org", "index.org"];

impl Website {
    pub fn load(path: &str) -> Result<Self, LoadError> {
        let path = Path::new(path);

        // for now, there's only one project, the blog, that simply collects
        // all posts in all folders of the website. In the longterm, rework
        // this so that it gets a foldername as input and uses only it.
        let mut projects = HashMap::new();
        projects.insert(String::from("blog"), Project::load(path)?);

        let mut pages = HashMap::new();
        for file in path.read_dir()? {
            let path = file?.path();
            let filename = path.file_name().unwrap().to_str().unwrap();

            if !path.is_file()
                || path.extension().unwrap() != "org"
                || IGNORED_FILES.contains(&filename)
            {
                continue;
            }
            let org = OrgFile::load(&path)?;
            pages.insert(org.path.clone(), org);
        }

        let mut links = HashMap::new();
        for proj in projects.values() {
            for post in proj.posts.values() {
                links.insert(post.path.as_path(), post);
            }
        }

        Ok(Website { projects, pages })
    }

    pub fn resolve_path(&self, path: &Path) -> Option<&OrgFile> {
        if let Some(page) = self.pages.get(path) {
            return Some(page);
        }

        for proj in self.projects.values() {
            if let Some(post) = proj.posts.get(path) {
                return Some(post);
            }
        }

        None
    }

    pub fn page_by_id(&self, id: &str) -> Option<&OrgFile> {
        for page in self.pages.values() {
            if page.id() == id {
                return Some(&page);
            }
        }
        None
    }
}

fn find_all_project_files(path: &Path) -> Result<Vec<PathBuf>, IOError> {
    let mut files = Vec::new();
    let mut folders = Vec::new();

    // load all folders in website directory
    for path in path.read_dir()? {
        let path = path?.path();
        let filename = path.file_name().unwrap().to_str().unwrap();
        if path.is_dir() && !IGNORED_FOLDERS.contains(&filename) {
            folders.push(path);
        }
    }

    // iterate over all folders and find files
    while folders.len() > 0 {
        let path = folders.pop().unwrap();
        for path in path.read_dir()? {
            let path = path?.path();
            if path.is_dir() {
                folders.push(path);
            } else {
                files.push(path);
            }
        }
    }

    Ok(files)
}

impl Project {
    fn load(path: &Path) -> Result<Self, LoadError> {
        // for now, there's only one project, the blog, that simply collects
        // all posts in all folders of the website. In the longterm, rework
        // this so that it gets a foldername as input and uses only it.

        let mut posts = HashMap::new();
        let mut ids = HashSet::new();
        for path in find_all_project_files(path)?.iter() {
            if path.file_name().unwrap() == "index.org" ||
                path.extension().is_none() ||
                path.extension().unwrap() != "org" {
                continue;
            }
            let org = OrgFile::load(path)?;
            if ids.contains(org.id()) {
                return Err(LoadError::DuplicateFileID(org.id().to_string()));
            }
            ids.insert(org.id().to_string());

            posts.insert(org.path.clone(), org);
        }

        Ok(Project { posts })
    }

    pub fn id(&self) -> &str {
        "blog"
    }
}

impl OrgFile {
    fn load(path: &PathBuf) -> Result<Self, LoadError> {
        let ext = path.extension();
        if ext.is_none() || ext.unwrap() != "org" {
            panic!(
                "Trying to load {:?} as orgfile. This shouldn't happen",
                path
            );
        }

        let contents = String::from_utf8(fs::read(path)?)?;
        let parser = Org::parse(&contents);

        let preamble = OrgFile::extract_preamble(&parser, path);
        let published = match preamble.get("published") {
            None => None,
            Some(d) => Some(OrgFile::parse_date(&d)?),
        };
        let last_edit = match preamble.get("last-edit") {
            None => None,
            Some(d) => Some(OrgFile::parse_date(&d)?),
        };

        Ok(OrgFile {
            id: path.file_stem().unwrap().to_str().unwrap().to_string(),
            path: path.clone(),
            contents,
            preamble,
            published,
            last_edit,
        })
    }

    fn extract_preamble(org: &Org, filename: &Path) -> HashMap<String, String> {
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
                        println!(
                            "Warning: encountered empty keyword '{}' while parsing org file {:?}",
                            k.key, filename
                        );
                    } else {
                        preamble.insert(k.key.to_string().to_lowercase(), k.value.to_string());
                    }
                }
                Some(Event::Start(_)) => break,
            };
        }
        preamble
    }

    fn parse_date(date_str: &str) -> chrono::ParseResult<chrono::naive::NaiveDate> {
        chrono::naive::NaiveDate::parse_from_str(date_str, "<%Y-%m-%d>")
    }

    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn from_preamble<'a>(&'a self, key: &str) -> Option<&'a str> {
        return self.preamble.get(key).and_then(|s| Some(s.as_str()));
    }

    pub fn resolve_link(&self, link: &str) -> PathBuf {
        let mut path = self.path.clone();
        path.pop();

        for part in Path::new(link) {
            match part.to_str().unwrap() {
                "." => {}
                ".." => {
                    path.pop();
                }
                part => path.push(part),
            }
        }

        path
    }
}

impl BlogElement for Website {
    fn url(&self, _website: &Website, base: String) -> String {
        base
    }

    fn title(&self) -> &str {
        "Johannes Huwald"
    }

    fn description(&self) -> &str {
        "My personal website"
    }
}

impl BlogElement for Project {
    fn url(&self, _website: &Website, base: String) -> String {
        base + "/" + self.id()
    }

    fn title(&self) -> &str {
        "Blog"
    }

    fn description(&self) -> &str {
        "Stuff I have written"
    }
}

impl BlogElement for OrgFile {
    fn url(&self, website: &Website, base: String) -> String {
        if website.pages.contains_key(&self.path) {
            return base + "/" + self.id();
        }

        for proj in website.projects.values() {
            if proj.posts.contains_key(&self.path) {
                return base + "/" + proj.id() + "/" + self.id();
            }
        }
        panic!("OrgFile:url called with Website that didn't load Orgfile");
    }

    fn title(&self) -> &str {
        let title = &self.from_preamble("title");
        assert!(
            title.is_some(),
            "Orgfile {:?} is missing a title",
            self.path
        );
        return title.unwrap();
    }

    fn description(&self) -> &str {
        self.from_preamble("summary").unwrap()
    }
}
