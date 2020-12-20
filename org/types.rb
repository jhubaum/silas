class OrgParseError < ::StandardError
end

class OrgReadFileError < OrgParseError
end

class InvalidTokenError < OrgParseError
end

class OrgDirectory
  attr_reader :files, :directories, :name, :path

  def initialize path, subfolder=nil
    raise OrgReadFileError, "Path given for OrgDirectory is no directory" unless Dir.exist? path

    if subfolder == nil
      @path = path
      @name = path.split("/").last
    else
      @path = path + "/" + subfolder
      @name = subfolder
    end

    @files = { }
    @directories = { }

    Dir.glob("#{@path}/*") do |f|
      name = f.split("/").last
      if File.file? f
        @files[name.split(".").first] = OrgParser.parse_file f
      else
        @directories[name] = OrgDirectory.new path, name
      end
    end
  end

  def all_files
    @files.each { |name, file| yield name, file }
    @directories.each do |d|
      d.all_files { |name, file| yield name, file }
    end
  end
end

class OrgFile
  private_class_method :new
  attr_reader :preamble, :elements


  def initialize preamble, elements
    @preamble, @elements = preamble, elements
  end

  def iterate_elements &block
    @elements.each do |elem|
      block.call elem
      if elem.respond_to? :iterate_elements
        elem.iterate_elements { |e| block.call e }
      end
    end
  end

  def to_html context
    @elements.to_html context, "\n"
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

class Section
  attr_reader :level, :title, :elements, :id

  def initialize level, title, elements, properties
    @level = level
    @title = title
    @elements = elements
    if properties.key? "CUSTOM_ID"
      @id = properties["CUSTOM_ID"]
    else
      @id = title.downcase.gsub(" ", "-")
    end
  end

  def heading
    "<h#{@level+2} id=\"#{@id}\">#{@title}</h#{@level+2}>"
  end

  def to_html context
    heading + "\n" + @elements.to_html(context, "\n")
  end
end

class Paragraph
  attr_reader :elements

  def initialize elements
    @elements = elements
  end

  def to_html context
    "<p>#{@elements.to_html context}</p>"
  end
end

class Block
  attr_reader :elements

  def initialize elements
    @elements = elements
  end

  def to_html context
    "<div class=\"#{class_name}\">#{@elements.to_html context}</div>"
  end
end

class Comment < Block
  def initialize elements
    super elements
  end

  def class_name
    "comment-block"
  end
end

class Quote < Block
  attr_reader :quotee

  def initialize elements, quotee
    super elements
    @quotee = quotee
  end

  def to_html context
    "<blockquote>" +
      "<p>#{@elements.to_html context}</p>" +
      (@quotee == nil ? "" : "<p>– #{@quotee.to_html context}<p>") +
    "</blockquote>"
  end
end

class SpecialText
  attr_accessor :text
  attr_reader :kind

  def initialize kind, text=""
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
    self[0].upcase + self[1..-1].downcase
  end

  def snakecase
    downcase.gsub(/[ -]/, "_")
  end
end

class Array
  def to_html context, div=""
    map { |e| e.to_html context }.join(div)
  end
end

class Link
  attr_accessor :attributes, :target, :text

  def initialize target, text
    @target, @text = target, text
    @attributes = []
  end

  def to_s
    @text == nil ? @target : @text
  end

  def to_html context
    target = context.resolve_link_target @target
    text = @text == nil ? target : @text
    style = @attributes.length > 0 ? "style=\"#{@attributes[0].style}\"" : ""

    "<a href=\"#{target}\" target=\"_blank\" #{style}>#{text}</a>"
  end
end

class HTMLStyleAttribute
  attr_reader :style
  def initialize str
    @style = str
  end
end
