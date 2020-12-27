require 'erb'
require 'tilt'

class Renderer
  attr_accessor :url_base

  def initialize builder, path
    @builder = builder
    @path = path

    # templates
    @layout = Tilt.new("theme/layout.html.erb")
    @post = Tilt.new("theme/post.html.erb")
    @page = Tilt.new("theme/page.html.erb")
    @project = Tilt.new("theme/project.html.erb")

    @url_base = nil
  end

  def post file
    puts "Render post #{file.path}"
    render @post, file, file
  end

  def page file
    puts "Render page #{file.path}"
    render @page, file, file
  end

  def project_index project
    puts "Render #{project.name}"
    render @project, project.index, project
  end

  private
  def render template, file, context
    return if file.draft? and not Config.preview
    path = file.url @path
    Dir.mkdir path unless Dir.exist? path
    File.open(path + "/index.html", "w+") do |f|
      f.write (@layout.render(context) do
                 template.render(context) { file.to_html @url_base}
               end)
    end
  end
end
