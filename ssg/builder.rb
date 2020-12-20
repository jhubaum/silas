require_relative 'project'
require_relative 'renderer'

require_relative '../org/parser'

class WebsiteBuilder
  def initialize path
    @orgdir = OrgDirectory.new path

    @pages = {}
    @projects = {}

    @orgdir.files.each do |name, file|
      @pages[name.snakecase.to_sym] = file unless name == "ideas"
    end

    @orgdir.directories.each do  |name, dir|
      @projects[name.snakecase.to_sym] = Project.new dir
    end
  end

  def header
    [
      Link.new(@pages[:about], "About"),
      @projects[:blog].create_link,
      @projects[:writing_fiction].create_link
    ]
  end

  def generate path
    r = Renderer.new self
    @pages.each { |sym, file| r.page file, "#{path}/#{sym}" }
    @projects.values.each do |proj|
      p = proj.url(path)
      r.page proj.index, p

      proj.files.each do |name, file|
        r.post file, "#{p}/#{name}"
      end
    end
  end

  def resolve_link target
    case target
    when OrgFile
      "This is a link to a file"
    when String
      target
    else
      raise "Don't know how to interpret target of type #{target.class}"
    end
  end
end
