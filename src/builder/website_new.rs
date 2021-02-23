use std::collections::{HashMap, HashSet};
use std::io::Error as IOError;
use std::path::{Path, PathBuf};

#[derive(Debug)]
pub enum LoadError {
    IO(IOError),
    DuplicateFileID(String)
}

impl From<IOError> for LoadError {
    fn from(err: IOError) -> Self {
        Self::IO(err)
    }
}

pub struct Website {
    pub projects: HashMap<String, Project>,
    pub pages: HashMap<PathBuf, OrgFile>,
}

pub struct Project {
    pub posts: HashMap<PathBuf, OrgFile>
}

#[derive(Clone)]
pub struct OrgFile {
    id: String,
    path: PathBuf
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

            if !path.is_file() ||
                path.extension().unwrap() != "org" ||
                IGNORED_FILES.contains(&filename) {
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

    pub fn file_url(&self, file: &OrgFile, basename: String) -> String {
        if self.pages.contains_key(&file.path) {
            return basename + "/" + file.id();
        }

        for proj in self.projects.values() {
            if proj.posts.contains_key(&file.path) {
                return basename + "/" + proj.id() + "/" + file.id();
            }
        }
        panic!("Website:file_url called with Orgfile not loaded by Website");
    }

    pub fn project_url(&self, project: &Project, basename: String) -> String {
        return basename + "/" + project.id()
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


        Ok (Project { posts })
    }

    pub fn id(&self) -> &str {
        "blog"
    }
}


impl OrgFile {
    fn load(path: &PathBuf) -> Result<Self, LoadError> {
        let ext = path.extension();
        if ext.is_none() || ext.unwrap() != "org" {
            panic!("Trying to load {:?} as orgfile. This shouldn't happen", path);
        }

        Ok ( OrgFile {
            id: path.file_stem().unwrap().to_str().unwrap().to_string(),
            path: path.clone()
        } )
    }

    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn resolve_link(&self, link: &str) -> PathBuf {
        let mut path = self.path.clone();
        path.pop();

        for part in Path::new(link) {
            match part.to_str().unwrap() {
                "." => {  },
                ".." => { path.pop(); },
                part => path.push(part)
            }
        }

        path
    }

}
