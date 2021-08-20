use std::collections::{HashMap, HashSet};
use std::io::Error as IOError;
use std::path::{Path, PathBuf};
use std::string::FromUtf8Error;

use orgize::{Element, Event, Org};
use std::fs;

use super::Mode;

#[derive(Debug)]
pub enum WebsiteError {
    IO(IOError),
    Page(PathBuf, OrgFileError),
    Project(String, ProjectError),
    DefaultProjectDoesNotExist,
}

#[derive(Debug)]
pub enum ProjectError {
    IO(IOError),
    DuplicateFileID(String),
    OrgFile(String, OrgFileError),
    UnknownProjectType,
}

#[derive(Debug)]
pub enum OrgFileError {
    IO(IOError),
    UTF8(FromUtf8Error),
    Date(chrono::ParseError),
    MissingRequiredField(&'static str),
}

impl From<IOError> for WebsiteError {
    fn from(err: IOError) -> Self {
        Self::IO(err)
    }
}

impl From<IOError> for ProjectError {
    fn from(err: IOError) -> Self {
        Self::IO(err)
    }
}

impl From<IOError> for OrgFileError {
    fn from(err: IOError) -> Self {
        Self::IO(err)
    }
}

impl From<FromUtf8Error> for OrgFileError {
    fn from(err: FromUtf8Error) -> Self {
        Self::UTF8(err)
    }
}

impl From<chrono::ParseError> for OrgFileError {
    fn from(err: chrono::ParseError) -> Self {
        Self::Date(err)
    }
}

impl OrgFileError {
    pub fn to_project_error<T>(res: Result<T, Self>, path: &Path) -> Result<T, ProjectError> {
        res.or_else(|err| {
            Err(ProjectError::OrgFile(
                path.to_str().unwrap().to_string(),
                err,
            ))
        })
    }
}

pub struct Website {
    pub projects: HashMap<String, Project>,
    pub pages: HashMap<PathBuf, OrgFile>,
    pub index: OrgFile,
}

pub struct Project {
    pub posts: HashMap<PathBuf, OrgFile>,
    id: String,
    pub index: OrgFile,
    pub project_type: ProjectType,
}

#[derive(Copy, Clone, PartialEq)]
pub enum ProjectType {
    /// A list of posts (like I'm using for my general blog). The default value
    Catalogue,
    /// Interpret the index file as a normal post but append an ordered list of all posts from the project
    MultiPart,
}

impl ProjectType {
    fn from_str(string: Option<&str>) -> Result<Self, ()> {
        if let Some(string) = string {
            return match string {
                "catalogue" => Ok(Self::Catalogue),
                "multi" => Ok(Self::MultiPart),
                _ => Err(()),
            };
        }
        Ok(Self::Catalogue)
    }
}

#[derive(Copy, Clone, PartialEq)]
pub enum PostType {
    /// A normal post, requiring a summary and subtitle if published
    Normal,
    /// A mini post. All posts in a multi part project are counted as Mini posts
    Mini,
    /// The type used for project indices
    Index,
    /// The type for all pages (and the website index)
    Page,
}

/// The order in which posts are rendered in the project index
#[derive(Debug)]
pub enum PostOrder {
    /// First, the The newest posts will be shown first. The default value
    NewestFirst,
    ById,
}

impl std::str::FromStr for PostOrder {
    type Err = ();

    fn from_str(string: &str) -> Result<Self, Self::Err> {
        match string {
            "newest" => Ok(Self::NewestFirst),
            "id" => Ok(Self::ById),
            _ => Err(()),
        }
    }
}

impl Default for PostOrder {
    fn default() -> Self {
        PostOrder::NewestFirst
    }
}

#[derive(Clone)]
pub struct OrgFile {
    id: String,
    preamble: HashMap<String, String>,
    pub path: PathBuf,
    pub contents: String,
    pub published: Option<chrono::naive::NaiveDate>,
    pub last_edit: Option<chrono::naive::NaiveDate>,
    // TODO: Add an intermediate struct Post that holds PostType instead
    pub post_type: PostType,
}

pub trait BlogElement {
    fn url(&self, website: &Website, base: String) -> String;
    fn title(&self) -> &str;
    fn description(&self) -> &str;
}

#[derive(Default)]
struct ProjectBuilder {
    posts: HashMap<PathBuf, OrgFile>,
    projects: HashMap<String, Project>,
}

impl Website {
    pub fn load<TMode: Mode>(path: &str) -> Result<Self, WebsiteError> {
        let path = Path::new(path);

        let mut project_builder = ProjectBuilder::default();

        let mut pages = HashMap::new();
        let mut index = None;
        for file in path.read_dir()? {
            let path = file?.path();
            let filename = path.file_name().unwrap().to_str().unwrap();

            if OrgFile::is_org_file(&path) {
                let org = OrgFile::load(&path, PostType::Page)
                    .or_else(|err| Err(WebsiteError::Page((&path).into(), err)))?;
                if path.file_name().unwrap() == "index.org" {
                    index = Some(org);
                } else if TMode::include_page(&org)
                    .or_else(|err| Err(WebsiteError::Page((&path).into(), err)))?
                {
                    pages.insert(org.path.clone(), org);
                }
            } else if path.is_dir() {
                project_builder.process_folder::<TMode>(&filename, &path)?;
            }
        }

        let index = index.expect("Found no website index (index.org in root directiory)");

        Ok(Website {
            projects: project_builder.projects(index.from_preamble("default_project").unwrap_or("blog"))?,
            pages,
            index
        })
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

impl ProjectBuilder {
    fn projects(mut self, default_project: &str) -> Result<HashMap<String, Project>, WebsiteError> {
        match self.projects.get_mut(default_project) {
            None => Err(WebsiteError::DefaultProjectDoesNotExist),
            Some(p) => {
                p.posts.extend(self.posts);
                Ok(self.projects)
            }
        }
    }

    fn process_folder<TMode: Mode>(&mut self, name: &str, path: &Path) -> Result<(), WebsiteError> {
        let mut index = path.to_path_buf();
        index.push("index.org");
        if index.exists() {
            self.projects.insert(
                name.to_string(),
                Project::load::<TMode>(name, path)
                    .or_else(|err| Err(WebsiteError::Project(name.to_string(), err)))?,
            );
        } else {
            for file in find_all_project_files(path)?.iter() {
                // TODO:
                // Setting the post type here doesn't work if the default project is a multi part project
                // In which the type should be PostType::Mini
                let org = OrgFile::load(&file, PostType::Normal)
                    .or_else(|err| Err(WebsiteError::Page(file.into(), err)))?;

                if TMode::include_post(&org)
                    .or_else(|err| Err(WebsiteError::Page(file.into(), err)))?
                {
                    self.posts.insert(file.to_path_buf(), org);
                }
            }
        }
        Ok(())
    }
}

fn find_all_project_files(path: &Path) -> Result<Vec<PathBuf>, IOError> {
    let mut files = Vec::new();
    let mut folders = Vec::new();
    folders.push(path.to_path_buf());

    while folders.len() > 0 {
        let path = folders.pop().unwrap();
        for path in path.read_dir()? {
            let path = path?.path();
            if path.is_dir() {
                folders.push(path);
            } else if OrgFile::is_org_file(&path) && path.file_name().unwrap() != "index.org" {
                files.push(path);
            }
        }
    }

    Ok(files)
}

impl Project {
    fn load<TMode: Mode>(id: &str, path: &Path) -> Result<Self, ProjectError> {
        let mut index = path.to_path_buf();
        index.push("index.org");
        let index = OrgFile::load(&index, PostType::Index);
        if index.is_err() {
            return Err(ProjectError::OrgFile(
                path.to_str().unwrap().to_string() + "/index.org",
                index.err().unwrap(),
            ));
        }
        let index = index.unwrap();

        let project_type = ProjectType::from_str(index.from_preamble("type"));

        if project_type.is_err() {
            return Err(ProjectError::UnknownProjectType);
        }
        let project_type = project_type.unwrap();

        let post_type = if project_type == ProjectType::MultiPart {
            PostType::Mini
        } else {
            PostType::Normal
        };

        let mut posts = HashMap::new();
        let mut ids = HashSet::new();
        for path in find_all_project_files(path)?.iter() {
            let org = OrgFileError::to_project_error(OrgFile::load(path, post_type), path)?;
            if ids.contains(org.id()) {
                return Err(ProjectError::DuplicateFileID(org.id().to_string()));
            }
            ids.insert(org.id().to_string());

            if OrgFileError::to_project_error(TMode::include_post(&org), path)? {
                posts.insert(org.path.clone(), org);
            }
        }

        Ok(Project {
            id: id.to_string(),
            project_type,
            posts,
            index,
        })
    }

    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn published(&self) -> bool {
        self.index.published.is_some()
    }
}

impl OrgFile {
    fn is_org_file(path: &Path) -> bool {
        path.is_file() && path.extension().map_or(false, |ext| ext == "org")
    }

    fn load(path: &PathBuf, post_type: PostType) -> Result<Self, OrgFileError> {
        assert!(
            OrgFile::is_org_file(path),
            "Trying to load {:?} as orgfile. This shouldn't happen",
            path
        );

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
            post_type,
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

    pub fn parse_from_preamble<T: std::str::FromStr + std::fmt::Debug>(
        &self,
        key: &str,
    ) -> Option<T>
    where
        T::Err: std::fmt::Debug,
    {
        self.from_preamble(key).map(|val| {
            let res = val.parse::<T>();
            assert!(
                res.is_ok(),
                "Unable to convert `{}` to {} in preamble from {:?}",
                val,
                std::any::type_name::<T>(),
                self.path
            );
            res.unwrap()
        })
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
        self.index.title()
    }

    fn description(&self) -> &str {
        self.index.title()
    }
}

impl BlogElement for Project {
    fn url(&self, _website: &Website, base: String) -> String {
        base + "/" + self.id()
    }

    fn title(&self) -> &str {
        self.index.title()
    }

    fn description(&self) -> &str {
        self.index.description()
    }
}

impl BlogElement for OrgFile {
    fn url(&self, website: &Website, base: String) -> String {
        if website.index.path == self.path {
            return base;
        }

        if website.pages.contains_key(&self.path) {
            return base + "/" + self.id();
        }

        for proj in website.projects.values() {
            if proj.index.path == self.path {
                return base + "/" + proj.id();
            }

            if proj.posts.contains_key(&self.path) {
                return base + "/" + proj.id() + "/" + self.id();
            }
        }
        panic!("OrgFile:url called on element not loaded by given website");
    }

    fn title(&self) -> &str {
        let title = &self.from_preamble("title");
        assert!(
            title.is_some(),
            "Orgfile {:?} is missing a title",
            self.path
        );
        title.unwrap()
    }

    fn description(&self) -> &str {
        let summary = self.from_preamble("summary");
        assert!(
            summary.is_some(),
            "Orgfile {:?} is missing a summary",
            self.path
        );
        summary.unwrap()
    }
}
