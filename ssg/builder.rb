require "pathname"

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
  IGNORED_FOLDERS = ["images"]
  IGNORED_ORG_FILES = ["ideas.org"]
  attr_reader :pages, :projects, :path, :index

  def initialize path
    @path = Pathname.new path
    @index = IndexOrgFile.new self
    @pages, @projects = {}, {}

    @path.children.each do |f|
      if f.non_index_org_file? and not IGNORED_ORG_FILES.include? f.basename
        @pages[f.realpath] = OrgFile.new f, self
      elsif f.directory? and not IGNORED_FOLDERS.include? f
        @projects[f.realpath] = Project.new f, self
      end
    end
  end

  def elements
    [@index] + @pages.values.to_a + @projects.values.to_a
  end

  def url path=nil
    path == nil ? @index.preamble[:url] : path
  end

  def add_external_file path
    path
  end

  def find_org_file path
    path = path.realpath
    return @pages[path] if @pages.key? path

    @projects.each do |p|
      return p.files[path] if p.files.key? path
    end

    raise "Link to invalid org file (Probably a parsing error)"
  end

  def add_and_get_dependency dependency
    raise "#{dependency} points to a path outside of website directory" unless @path.contains? dependency

    case
    when dependency == @path
      self
    when dependency.org_file?
      find_org_file dependency
    else
      add_external_file dependency
    end
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
    return []
    [
      #Link.new(nil, @website.pages[:about], "About"),
      #@website.projects[:blog].create_link,
      #@website.projects[:writing_fiction].create_link
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
    puts "This resolve link is obsolete"
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
        @website.resolve_path target[5..-1]
      else
        raise "Unable to deduce link type for target #{target}"
      end
    else
      raise "Don't know how to interpret target of type #{target.class}"
    end
  end
end
