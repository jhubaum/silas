require "pathname"
require "set"
require_relative "../org/types"

class Project < OrgObject
  attr_reader :index, :files, :path

  def initialize path, website
    @website = website
    @path = path
    @index = IndexOrgFile.new self
    @files = populate_files
    @external_files = {}
  end

  def external_files
    return @external_files.values
  end

  def parent
    @website
  end

  def relative_path
    @path.relative_path_from @website.path
  end

  def name
    path.basename.to_s.titlecase
  end

  def id
    path.basename.to_s.snakecase
  end

  def url path=nil
    "#{@website.url path}/#{id}"
  end

  def elements
    [@index] + @files.values.to_a
  end

  def posts
    @files.values.sort do |lhs, rhs|
      lhs = lhs.info.published
      rhs = rhs.info.published

      lhs == nil ? -1 : (rhs == nil ? 1 : -(lhs <=> rhs))
    end
  end

  def create_link
    Link.new nil, self, name
  end

  def draft?
    @index.draft?
  end

  def resolve_path path
    split = path.split("/")
    @website.resolve_path(split.first == ".." ? split.tail.join("/") : path)
  end

  def visit visitor
    @index.visit visitor
    @files.values.each { |f| f.visit visitor }
  end

  def add_external_file path
    @external_files[path.realpath] = ExternalFile.new path, self unless @external_files.key? path.realpath

    @external_files[path.realpath]
  end

  def add_and_get_dependency dependency
    return @website.add_and_get_dependency(dependency) unless @path.contains? dependency

    case
    when dependency == @path, dependency == @index.path
      self
    when dependency.org_file?
      files[dependency.realpath]
    else
      add_external_file dependency
    end
  end

  private
  def populate_files
    files = { }
    ids = Set.new

    Pathname.all_files_recursively(path).each do |f|
      if f.non_index_org_file? and f.filename != "ideas"
        file = OrgFile.new f, self
        files[f.realpath] = file

        raise "duplicate id '#{file.id}' (in project '#{name}')" if ids.member? file.id
        ids << file.id
      end
    end
    files
  end

  def css_path filename
    "../" + @website.send(:css_path, filename)
  end
end
