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
  class Context
    def initialize file
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
    c = Context.new file
    Dir.mkdir path
    File.open(path + "/index.html", "w+") do |f|
      f.write (@layout.render(c, :header => @builder.header) do
                 template.render(c) { file.to_html c}
               end)
    end
  end
end
