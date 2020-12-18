require_relative 'types'

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

  def generate path
    r = Renderer.new "https://jhuwald.com"
    @pages.each do |sym, file|
      puts "Render #{sym}"
      p = "#{path}/#{sym}"
      Dir.mkdir p
      File.open(p + "/index.html", "w+") do |f|
        f.write r.post(file)
      #render_post file, "#{path}/#{sym}"
      end
    end
  end

  private
  def add_page path
    return if path == "ideas"

    puts "Add page #{path}"
    @pages[path.to_sym] = OrgParser.parse_file "#{@path}/#{path}.org"
  end

  def add_project path
    puts "Add project #{path}"
    @projects[path.to_sym] = Project.new @path, path
  end
end
