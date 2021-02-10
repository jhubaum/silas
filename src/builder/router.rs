use super::website::{Website, Project, Post};

pub trait Router {
    fn project_url(&self, project: &Project, base: String) -> String;
    fn post_url(&self, post: &Post, base: String) -> String;

    fn css_path_for_post(&self, post: &Post, css: &str) -> String;
}

pub struct SingleBlogFolderRouter<'website> {
    pub website: &'website Website
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
