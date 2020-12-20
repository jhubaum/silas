require 'erb'
require 'tilt'

class Renderer
  class Context
    def initialize renderer, file
      @renderer = renderer
      @title = file.preamble[:title]
      @published = file.preamble[:published]
      @last_edit = file.preamble[:lastedit]
    end

    def published?
      @published != nil
    end

    def last_edit?
      @last_edit != nil
    end

    def resolve_link_target target
      return nil if target == nil
      @renderer.builder.resolve_link target
    end
  end

  attr_reader :builder

  def initialize builder
    @builder = builder
    @layout = Tilt.new("theme/layout.html.erb")
    @post = Tilt.new("theme/post.html.erb")
    @page = Tilt.new("theme/page.html.erb")
  end

  def post file, path
    render @post, file, path
  end

  def page file, path
    render @page, file, path
  end

  private
  def render template, file, path
    c = Context.new self, file
    Dir.mkdir path
    File.open(path + "/index.html", "w+") do |f|
      f.write (@layout.render(c, :header => @builder.header) do
                 template.render(c) { file.to_html c}
               end)
    end
  end
end
