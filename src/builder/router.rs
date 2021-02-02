use super::website::{Website, Project, Post};

pub trait Router {
    fn project_url(&self, project: &Project, base: String) -> String;
    fn post_url(&self, post: &Post, base: String) -> String;
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
            None => base + "/" + &post.id,
            Some(_) => base + "/blog/" + &post.id
        }
    }
}
