require "pathname"
require "fileutils"
require "rouge"

class OrgParseError < ::StandardError
end

class OrgReadFileError < OrgParseError
end

class InvalidTokenError < OrgParseError
end

class OrgObject
  def visit visitor
    method_name = "visit_#{self.class}"
    visitor.send(method_name, self) if visitor.respond_to? method_name

    if respond_to? :elements
      elements.each do |e|
        if e.is_a? OrgObject
          e.visit visitor
        else
          method_name = "visit_#{e.class}"
          visitor.send(method_name, e) if visitor.respond_to? method_name
        end
      end
    end
  end

  def draft?
    false
  end

  def iterate include_self=false
    yield self if include_self

    if respond_to? :elements
      elements.each do |e|
        yield e
        e.iterate(false) { yield } if e.is_a? OrgObject
      end
    end
  end

  def css filename
    "<link href=\"#{send :css_path, filename}\"  rel=\"stylesheet\" type=\"text/css\">"
  end
end

class ExternalFile < OrgObject
  def initialize path, parent
    @parent = parent
    @path = path
  end

  def url path=nil
    "#{@parent.url path}/#{id}"
  end

  def id
    @path.basename
  end

  def copy path
    FileUtils.cp @path.realpath.to_s, url(path)
  end

  def type
    case @path.fileending
    when "jpg", "jpeg", "png"
      :image
    else
      raise "Unknown filetype for external file '#{@path}'"
    end
  end

  def of_type? val
    val == type
  end
end

class OrgTextObject
  attr_reader :file

  def initialize file
    @file = file
  end
end

class OrgFile < OrgObject
  attr_reader :info, :elements, :path, :parent

  def initialize path, parent=nil
    @parent = parent
    @path = path
    @info, @elements = OrgParser.parse_file self, path
  end

  def draft?
    @info.draft or @info.published == nil
  end

  def url path=nil
    @parent == nil ? id : "#{@parent.url path}/#{id}"
  end

  def relative_path
    @path.relative_path_from @parent.path
  end

  def id
    @path.filename.snakecase
  end

  def to_html context
    @elements.to_html context, "\n"
  end

  def add_and_get_dependency dependency
    @parent.add_and_get_dependency dependency
  end

  def resolve_path path
    path, section = path.split("::")
    path = Pathname.new path if path.instance_of? String
    path = @path.dirname + path

    raise "link in '#{@path}' points to invalid file #{path}" unless path.file?

    file = add_and_get_dependency path
    section == nil ? file : file.find_section(section)
  end

  def find_section section
    puts "Link to section '#{section}'"
    case section[0]
    when "*"
      sections { |s| return s if s.title == section[1..-1] }
    when "#"
      sections { |s| return s if s.id == section[1..-1] }
    else
      raise "Invalid link #{section}' to section"
    end
    nil
  end

  def sections
    iterate { |e| yield e if e.instance_of? Section }
  end

  private
  def css_path filename
    "../" + @parent.send(:css_path, filename)
  end
end

class IndexOrgFile < OrgFile
  def initialize parent
    super parent.path + Pathname.new("index.org"), parent
  end

  def draft?
    @info.draft
  end

  def url path=nil
    @parent.url path
  end
end

def print_element_tree object, indent = 0
  puts " " * indent + object.class.to_s
  if object.respond_to? :elements
    object.elements.each { |e| print_element_tree e, indent+2 }
  end
end

class Preamble
  attr_reader :title, :published, :last_edit, :render_type, :draft

  def initialize **values
    @title = values[:title]
    @published = Date.from_s values.fetch(:published, nil)
    @last_edit = Date.from_s values.fetch(:lastedit, nil)
    @draft = values.fetch(:draft, false)
    @render_type = values.fetch(:rendertype, :list).to_sym

    @values = values
  end

  def summary?
    @values.key? :summary
  end

  def summary
    get :summary
  end

  def get key
    @values.fetch key
  end
end


class Date
  attr_reader :year, :month, :day
  def Date.from_s s
    return nil if s == nil
    /<(?<y>\d{4})-(?<m>\d{2})-(?<d>\d{2})>/ =~ s
    Date.new y, m, d
  end

  def initialize year, month, day
    @year = year.to_i
    @month = month.to_i
    @day = day.to_i
  end

  def to_s
    "<#{@year}-#{@month}-#{@day}>"
  end

  MONTH_NAMES = ["January", "February", "March", "April",
                 "May", "June", "July", "August",
                 "September", "October", "November", "December"]

  def to_pretty_s
    "#{MONTH_NAMES[@month-1]} #{@day}, #{@year}"
  end

  def <=> rhs
    return -1 if rhs == nil
    res = @year <=> rhs.year
    res = @month <=> rhs.month if res == 0
    res = @day <=> rhs.day if res == 0
    res
  end
end

class Section < OrgTextObject
  attr_reader :level, :title, :elements, :id

  def initialize file, **args
    super file

    @level = args[:level]
    @title = args[:title]
    @elements = args[:elements]
    properties = args[:properties]
    @id = (properties.key? "CUSTOM_ID") ? properties["CUSTOM_ID"] :
            @title.downcase.gsub(" ", "-").gsub(/[^a-z0-9 ]/, "")
  end

  def heading
    "<h#{@level+2} id=\"#{@id}\">#{@title}</h#{@level+2}>"
  end

  def to_html context
    heading + "\n" + @elements.to_html(context, "\n")
  end

  def url base=nil
    "#{@file.url base}\##{@id}"
  end
end

class Paragraph < OrgTextObject
  attr_reader :elements

  def initialize file, elements
    super file

    @elements = elements
  end

  def to_html context
    "<p>#{@elements.to_html context}</p>"
  end
end

class Block < OrgTextObject
  attr_reader :elements

  def initialize file, elements
    super file

    @elements = elements
  end

  def to_html context
    "<div class=\"#{class_name}\">#{@elements.to_html context}</div>"
  end
end

class Comment < Block
  def initialize file, elements
    super file, elements
  end

  def class_name
    "comment-block"
  end
end

class Quote < Block
  attr_reader :quotee

  def initialize file, elements, quotee
    super file, elements
    @quotee = quotee
  end

  def to_html context
    "<blockquote>" +
      "<p>#{@elements.to_html context}</p>" +
      (@quotee == nil ? "" : "<cite>#{@quotee.to_html context}</cite>") +
    "</blockquote>"
  end
end

class CodeBlock < Block
  def initialize lang, code
    @@formatter ||= Rouge::Formatters::HTML.new

    @lexer = lexer lang
    @code = code
  end

  def lexer lang
    case lang
    when "python", "python3"
      Rouge::Lexers::Python.new
    else
      raise "Codeblock: Unknown language #{lang}"
    end
  end

  def to_html context
    code = @code.map { |c| convert_code_line c }.join("<br>\n")
    "<p class=\"codeblock\">\n#{code}\n</p>"
  end

  private
  def convert_code_line line
    @@formatter.format(@lexer.lex line).replace_leading_spaces_with "&nbsp;"
  end
end

class SpecialText < OrgTextObject
  attr_accessor :text
  attr_reader :kind

  def initialize file, kind, text=""
    super file

    @kind = kind
    @text = text
  end

  def text= text
    @text
  end

  def to_s
    return @text
  end

  def to_html context
    case @kind
    when :bold
      "<b>#{text}</b>"
    when :italic
      "<em>#{text}</em>"
    else
      raise ArgumentError, "Invalid kind #{kind} for SpecialText"
    end
  end
end

class String
  def to_html context
    self
  end

  def titlecase
    gsub(/[-_]/, " ").split(" ").map { |s| s[0].upcase + s[1..-1].downcase }.join " "
  end

  def snakecase
    downcase.gsub(/[ -]/, "_")
  end

  def replace_leading_spaces_with str
    /^ */ =~ self
    cnt = $~.to_s.length
    return str * cnt + self[cnt..-1]
  end

  def red
    colorize(31)
  end

  def green
    colorize(32)
  end

  def yellow
    colorize(33)
  end

  def blue
    colorize(34)
  end

  def pink
    colorize(35)
  end

  def light_blue
    colorize(36)
  end

  private
  def colorize(color_code)
    "\e[#{color_code}m#{self}\e[0m"
  end
end

class Pathname
  def fileending
    basename.to_s.split(".").last.downcase
  end

  def filename
    basename.to_s.split(".").first
  end

  def non_index_org_file?
    org_file? and not index_org_file?
  end

  def index_org_file?
    basename.to_s == "index.org"
  end

  def org_file?
    file? and fileending == "org"
  end

  def contains? path
    not path.relative_path_from(self).to_s.start_with? ".."
  end

  def Pathname.all_files_recursively path
    Pathname.glob("#{path}/**/*.*")
  end
end

class Array
  def to_html context, div=" "
    map { |e| e.to_html context }.join(div)
  end

  def head
    first
  end

  def tail
    self[1..-1]
  end
end

class Link < OrgTextObject
  attr_accessor :attributes, :target, :text

  def initialize file, target, text
    super file

    @target, @text = target, text
    @attributes = []
  end

  def to_s
    @text == nil ? @target : @text
  end

  def resolve_target!
    return @target unless @target.instance_of? String
    puts "A link has target #{@target}"

    case @target.split(":").first
    when "http", "https", "mailto"
      # link is ok
    when "file"
      @target = @file.resolve_path target[5..-1]
    else
      raise "Unable to deduce link type for target #{target}"
    end

    puts "Resolved link target to #{@target}"
    @target
  end

  def to_html context
    resolve_target!
    if @target.is_a? OrgObject and @target.draft?
      puts "Warning: Link #{@target.path} in #{@file.path} points to draft".yellow
      return @text unless Config.preview
    end
    style = @attributes.length > 0 ? " style=\"#{@attributes[0].style}\"" : ""

    if @target.instance_of? ExternalFile and @target.of_type? :image
      text = @text == nil ? "" : "alt=#{@text}"
      "<img src=\"#{@target.url context}\"#{style}#{text}>"
    else
      target = (@target.respond_to? :url) ? @target.url(context) : @target
      text = @text == nil ? target : @text
      "<a href=\"#{target}\" target=\"_blank\"#{style}>#{text}</a>"
    end
  end
end

class HTMLStyleAttribute
  attr_reader :style
  def initialize str
    @style = str
  end
end

module ListType
  class Ordered
    def tag
      "ol"
    end

    def is_next_entry tok, count
      tok.value.to_i == count + 1
    end

    def indentation
      3
    end
  end

  class Unordered
    def initialize kind
      @kind = kind
    end

    def tag
      "ul"
    end

    def is_next_entry tok, count
      tok.is? @kind
    end

    def indentation
      2
    end
  end
end

class List < OrgTextObject
  def initialize file, type, entries
    super file
    @type = type
    @entries = entries
  end

  def elements
    @entries
  end

  def to_html context
    list = @entries.to_html context, "</li><li>"
    "<#{@type.tag}><li>#{list}</li></#{@type.tag}>"
  end
end
