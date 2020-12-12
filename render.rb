require 'erb'
require 'tilt'

module SSG
  def render_post file, path
    template = Tilt.new('templates/post.html.erb')
    File.open(path + "/index.html", "w+") do |f|
      f.write template.render(self, :test => "body content",
                              :title => "Example post")
    end
  end
end
