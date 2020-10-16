import os, shutil
import json
# https://jinja.palletsprojects.com/en/2.11.x/api/
from jinja2 import Environment, ChoiceLoader, PackageLoader, FileSystemLoader, select_autoescape


class URLObject:
    def __init__(self, url, name):
        self.url = url
        self.name = name

    def get_link(self, depth=2):
        return dict(
            url=("../" * depth) + self.url,
            name=self.name
        )


class Post(URLObject):
    @staticmethod
    def from_json(base_path, url, obj):
        return Post(url=url,
                    filepath=os.path.join(base_path, obj["path"]),
                    title=obj["title"], draft=obj.get("draft", False))

    def __init__(self, *, url, filepath, title, draft):
        super().__init__(url, title)
        self.template = filepath
        self.title = title
        self.draft = draft

    def args(self):
        return dict(
            title=self.title
        )


def update_category_config_file(path):
    c = os.path.join(path, "config.json")
    if not os.path.isfile(c):
        with open(c, 'w+') as f:
            f.write(json.dumps(dict()))


class Category(URLObject):
    @staticmethod
    def from_json(basepath, url, obj):
        base_url = f"category/{url}"
        path = obj["path"]

        update_category_config_file(os.path.join(basepath, path))

        with open(os.path.join(basepath, path, "config.json")) as f:
            posts = [Post.from_json(path, f"{base_url}/{url}", obj)
                     for url, obj in json.loads(f.read()).items()]

        return Category(url=base_url, name=obj["name"], posts=posts)

    def __init__(self, *, url, name, posts):
        super().__init__(url, name)
        self.posts = posts

    def args(self):
        return dict(
            title=f"Category - {self.name}",
            name=self.name,
            posts=self.posts
        )


def load_config(path):
    with open(os.path.join(os.path.join(path, "config.json"))) as f:
        config = json.loads(f.read())

    categories = [Category.from_json(path, url, obj)
                  for url, obj in config["categories"].items()]

    pages = dict(
        index=Post(url="",
                   filepath=config["pages"]["index"],
                   title=config["title"], draft=False)
    )
    return categories, pages




class SiteBuilder:
    def __init__(self, path):
        self.categories, self.pages = load_config(path)

        self.env = Environment(loader=ChoiceLoader([PackageLoader("silas", "theme"),
                                                    FileSystemLoader(path)]),
                               autoescape=select_autoescape(["html"]))

    def _render_template(self, template, obj, depth):
        p = os.path.join(self.export_path, obj.url)
        if not os.path.isdir(p):
            os.mkdir(p)
        with open(os.path.join(p, "index.html"), "w+") as f:
            template = self.env.get_template(template)
            f.write(template.render(**obj.args(),
                                    categories=[c.get_link(depth) for c in self.categories]))


    def build(self, export_path="generated",
              overwrite=True, indir=False):
        """
        Indir: if true, the project directory will be used as base path
        """
        if indir:
            export_path = os.path.join(os.path.dirname(__file__), export_path)

        self.export_path = export_path

        if overwrite and os.path.isdir(export_path):
            shutil.rmtree(export_path)

        os.mkdir(export_path)
        os.mkdir(os.path.join(export_path, "category"))

        for c in self.categories:
            self._render_template("category.html", c, 2)
            for p in c.posts:
                self._render_template(p.template, p, 3)

        for page in self.pages.values():
            self._render_template(page.template, page, 0)
