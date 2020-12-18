require 'erb'
require 'tilt'

module SSG
  def render_post file, path
    template = Tilt.new('theme/post.html.erb')
    Dir.mkdir path
    File.open(path + "/index.html", "w+") do |f|
      f.write template.render(file,
                              :pages => [
                                ["https://jhuwald.com/about/", "About"],
                                ["https://jhuwald.com/blog/", "Blog"],
                                ["https://jhuwald.com/projects/", "Projects"],
                                ["https://jhuwald.com/stories/", "Stories"]
                              ]) { file.to_html}
    end
  end
end

class Renderer
  def initialize website_name
    @name = website_name
    @layout = Tilt.new("theme/layout.html.erb")
    @post = Tilt.new("theme/post.html.erb")
    #@page = Tilt.new("theme/page.html.erb")
  end

  def post file
    @layout.render(file, :pages => []) do
      @post.render(file) { file.to_html }
    end
  end
end
