require 'erb'
require 'tilt'

module SSG
  def render_post file, path
    template = Tilt.new('templates/post.html.erb')
    File.open(path + "/index.html", "w+") do |f|
      f.write template.render(file, :title => "Example post",
                              :pages => [
                                ["https://jhuwald.com/about/", "About"],
                                ["https://jhuwald.com/blog/", "Blog"],
                                ["https://jhuwald.com/projects/", "Projects"],
                                ["https://jhuwald.com/stories/", "Stories"]
                              ])
    end
  end
end
