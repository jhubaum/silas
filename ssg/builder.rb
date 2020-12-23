require_relative 'project'
require_relative 'renderer'

require_relative '../org/parser'

=begin
All classes that represent something in the website have five methods:
- url: the complete url to the object. Example: https://jhuwald.com/writing_fiction/freedrafts
- id: the object related identifier. Example: freedrafts in url above
- path: The path relative to the website root.
  Example: writing_fiction/freedrafts.org
- filename: The filename. Example freedrafts.org
- name: The identifer for the object. Example: Freedrafts
=end

class Website < OrgObject
  IGNORE_FOLDERS = ["images", ".", ".."]
  attr_reader :pages, :projects, :path, :index

  def initialize path
    @path = path
    @index = IndexOrgFile.new self
    @pages, @projects = {}, {}

    Dir.entries(@path).each do |f|
      if File.file? File.join(path, f)
        name = f.split(".").first
        if f.non_index_org_file? and name != "ideas"
          @pages[name.snakecase.to_sym] = OrgFile.new f, self
        end
      elsif not IGNORE_FOLDERS.include? f
        @projects[f.snakecase.to_sym] = Project.new f, self
      end
    end
  end

  def path_to_url path
    path = path.split("/")

    if path.length == 0
      @pages.values.each { |p| return p.url if p.filename == path.first }
    else
      @projects.values.each { |p| return p.path_to_url path.tail.join("/") if p.filename == path.first }
    end
    nil
  end

  def elements
    [@index] + @pages.values.to_a + @projects.values.to_a
  end

  def url path=nil
    path == nil ? @index.preamble[:url] : path
  end
end

class ResolveLinksVisitor
  def initialize
    @file = nil
  end

  def visit_OrgFile file
    @file = file
  end

  def visit_OrgIndexFile file
    @file = file
  end

  def visit_Link link
    link.target = @file.resolve_relative_path link.target
  end
end

class WebsiteBuilder
  def initialize path
    @website = Website.new path
    @website.visit ResolveLinksVisitor.new
  end

  def header
    [
      Link.new(nil, @website.pages[:about], "About"),
      @website.projects[:blog].create_link,
      @website.projects[:writing_fiction].create_link
    ]
  end

  def generate path
    r = Renderer.new self, path
    r.page @website.index
    @website.pages.each { |sym, file| r.page file }
    @website.projects.values.each do |proj|
      r.page proj.index

      proj.files.each do |name, file|
        r.post file
      end
    end
  end

  def resolve_link target
    case target
    when Project, OrgFile
      target.url
    when OrgFile
      "This is a link to a file"
    when String
      type = target.split(":").first
      case type
      when "http", "https", "mailto"
        target
      when "file"
        @website.path_to_url target[5..-1]
      else
        raise "Unable to deduce link type for target #{target}"
      end
    else
      raise "Don't know how to interpret target of type #{target.class}"
    end
  end
end
