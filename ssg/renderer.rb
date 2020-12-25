require 'erb'
require 'tilt'

class Renderer
  class Context
    def initialize renderer, file
      @renderer = renderer
      @info = file.preamble
    end
  end

  attr_reader :builder

  def initialize builder, path
    @builder = builder
    @path = path

    # templates
    @layout = Tilt.new("theme/layout.html.erb")
    @post = Tilt.new("theme/post.html.erb")
    @page = Tilt.new("theme/page.html.erb")
  end

  def post file
    render @post, file
  end

  def page file
    render @page, file
  end

  private
  def render template, file
    path = file.url @path
    c = Context.new self, file
    Dir.mkdir path unless Dir.exist? path
    File.open(path + "/index.html", "w+") do |f|
      f.write (@layout.render(c, :header => @builder.header) do
                 template.render(c) { file.to_html c}
               end)
    end
  end
end
