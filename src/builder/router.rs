use super::website::{Website, Project, Post};

pub trait Router {
    fn project_url(&self, project: &Project, base: String) -> String;
    fn post_url(&self, post: &Post, base: String) -> String;

    fn css_path_for_post(&self, post: &Post, css: &str) -> String;
}

pub struct SingleBlogFolderRouter<'website> {
    pub website: &'website Website
}

pub struct NoopRouter {  }

impl Router for NoopRouter {
    fn project_url(&self, _project: &Project, _base: String) -> String {
        String::from("")
    }

    fn post_url(&self, _post: &Post, _base: String) -> String {
        String::from("")
    }

    fn css_path_for_post(&self, _post: &Post, _css: &str) -> String {
        String::from("")
    }
}

impl Router for SingleBlogFolderRouter<'_> {
    fn project_url(&self, project: &Project, base: String) -> String {
        base + &project.id
    }

    fn post_url(&self, post: &Post, base: String) -> String {
        match post.index.project {
            None => base + "/" + &post.id(),
            Some(_) => base + "/blog/" + &post.id()
        }
    }

    fn css_path_for_post(&self, post: &Post, css: &str) -> String {
        match post.index.project {
            None => String::from("../css/") + css,
            Some(_) => String::from("../../css/") + css
        }
    }
}
