class OrgParseError < ::StandardError
end

class OrgReadFileError < OrgParseError
end

class InvalidTokenError < OrgParseError
end

class OrgDirectory
  def initialize dirname
    @name = dirname
    @files = []
    raise OrgReadFileError, "Path given for OrgDirectory is no directory" unless Dir.exist? @name

    @files = Dir.glob("**/*.org", base: @name).map { |f| OrgFile.new @name, f }

    @files.each { |f| puts f.path }
  end
end

class OrgFile
  attr_reader :preamble, :elements

  def initialize dirname=nil, file_name
    @name = (dirname == nil) ? file_name : File.join(dirname, file_name)
    raise OrgReadFileError, "file '#{@name}' does not exist" unless File.file? @name

    @preamble = { }
    tokens = Tokenizer.tokenize File.open(@name).read

    #parse preamble
    while tokens.pop_if { |t| t.is? :preamble_start }
      val = tokens.pop_expected(:word).value
      tokens.pop_expected :colon
      tokens.pop_if { |t| t.is? :whitespace }
      elems = tokens.pop_until { |t| t.is? :newline }
      @preamble[val.to_sym] = tokens_to_s elems

      # remove newline
      tokens.pop
    end

    @preamble[:published] = OrgParsing.s_to_date @preamble[:published]

    tokens.pop_while { |t| t.is? :newline }

    #puts "Finished preamble (#{preamble.keys})"

    @elements = OrgParsing.parse tokens
  end

  def iterate_elements &block
    @elements.each do |elem|
      block.call elem
      if elem.respond_to? :iterate_elements
        elem.iterate_elements { |e| block.call e }
      end
    end
  end
end

def print_element_tree object, indent = 0
  puts " " * indent + object.class.to_s
  if object.respond_to? :elements
    object.elements.each { |e| print_element_tree e, indent+2 }
  end
end


class Date
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
    @elements=elements
    if properties.key? "CUSTOM_ID"
      @id = properties["CUSTOM_ID"]
    else
      @id = title.downcase.gsub(" ", "-")
    end
  end

  def append element
    @elements<< element
  end

  def heading
    "<h#{@level+2} id=\"#{@id}\">#{@title}</h#{@level+2}>"
  end

  def iterate_elements &block
    @elements.each do |elem|
      block.call elem
      if elem.respond_to? :iterate_elements
        elem.iterate_elements { |e| block.call e }
      end
    end
  end
end

class Paragraph
  attr_reader :elements

  def initialize elements
    @elements = elements
  end

  def to_html
    "<p>#{@elements.map(&:to_html).join(" ")}</p>"
  end
end

class Block
  attr_reader :elements

  def initialize elements
    @elements = elements
  end

  def to_html
    "<div class=\"#{class_name}\">#{@elements.map(&:to_html).join(" ")}</div>"
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
  attr_reader :text, :quotee

  def initialize text, quotee
    @text = text
    @quotee = quotee
  end

  def elements
    @text + @quotee
  end

  def to_html
    "<blockquote>" +
      "<p>#{@text.to_html}</p>" +
      (@quotee == nil ? "" : "<p>– #{@quotee.to_html}<p>") +
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

  def to_html
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
  def to_html
    self
  end
end

class Array
  def to_html
    map(&:to_html).join(" ")
  end
end

class Link
  def initialize target, text
    @target, @text = target, text
  end

  def to_html
    "<a href=\"#{@target}\" target=\"_blank\">#{@text == nil ? @target : @text}</a>"
  end
end

class HTMLStyleAttribute
  attr_reader :style
  def initialize str
    @style = str
  end
end
