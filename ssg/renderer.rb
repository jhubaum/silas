require 'erb'
require 'tilt'

class Renderer
  def initialize builder, path
    @builder = builder
    @path = path

    # templates
    @layout = Tilt.new("theme/layout.html.erb")
    @post = Tilt.new("theme/post.html.erb")
    @page = Tilt.new("theme/page.html.erb")
    @project = Tilt.new("theme/project.html.erb")
  end

  def post file
    render @post, file, file
  end

  def page file
    render @page, file, file
  end

  def project_index project
    render @project, project.index, project
  end

  private
  def render template, file, context
    path = file.url @path
    Dir.mkdir path unless Dir.exist? path
    File.open(path + "/index.html", "w+") do |f|
      f.write (@layout.render(context,
                              :header => @builder.header,
                              :title => file.info.title) do
                 template.render(context) { file.to_html nil}
               end)
    end
  end
end
