require_relative 'project'

require_relative '../org/parser'

class WebsiteBuilder
  def initialize path
    @path = path
    @pages = { }
    @projects = { }

    Dir.glob("#{path}/*").each do |f|
      name = f.split("/").last
      if File.file? f
        add_page name.split(".").first
      else
        add_project name
      end
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

  private
  def add_page path
    return if path == "ideas"
    @pages[path.to_sym] = OrgParser.parse_file "#{@path}/#{path}.org"
  end

  def add_project path
    @projects[path.to_sym] = Project.new @path, path
  end
end
