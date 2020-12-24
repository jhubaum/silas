require "pathname"
require "fileutils"

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
  attr_reader :preamble, :elements, :path, :parent

  def initialize path, parent=nil
    @parent = parent
    @path = path
    @preamble, @elements = OrgParser.parse_file self, path
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

  def name
    @preamble[:title]
  end

  def to_html context
    @elements.to_html context, "\n"
  end

  def add_and_get_dependency dependency
    @parent.add_and_get_dependency dependency
  end

  def resolve_path path
    # ignore link to sections for now
    path = path.split("::").first
    path = Pathname.new path if path.instance_of? String
    path = @path.dirname + path

    raise "link in '#{@path}' points to invalid file #{path}" unless path.file?

    add_and_get_dependency path
  end
end

class IndexOrgFile < OrgFile
  def initialize parent
    super parent.path + Pathname.new("index.org"), parent
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


class Date
  def Date.from_s s
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
            @title.downcase.gsub(" ", "-")
  end

  def heading
    "<h#{@level+2} id=\"#{@id}\">#{@title}</h#{@level+2}>"
  end

  def to_html context
    heading + "\n" + @elements.to_html(context, "\n")
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
      (@quotee == nil ? "" : "<p>– #{@quotee.to_html context}<p>") +
    "</blockquote>"
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

    case @target.split(":").first
    when "http", "https", "mailto"
      # link is ok
    when "file"
      @target = @file.resolve_path target[5..-1]
    else
      raise "Unable to deduce link type for target #{target}"
    end

    @target
  end

  def to_html context
    resolve_target!
    style = @attributes.length > 0 ? " style=\"#{@attributes[0].style}\"" : ""

    if @target.instance_of? ExternalFile and @target.of_type? :image
      text = @text == nil ? "" : "alt=#{@text}"
      "<img src=\"#{@target.url}\"#{style}#{text}>"
    else
      target = (@target.respond_to? :url) ? @target.url : @target
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
