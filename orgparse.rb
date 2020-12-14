require './tokenizer'
require './token_helpers'

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

module OrgParsing
  def OrgParsing.parse tokens, until_token=nil
    #puts "Enter Parse (until token: #{until_token})"
    elements = []
    attributes = []
    while tokens.has_tokens?
      token = tokens.peek
      return elements if token.is? until_token
      #puts "Enter parsing loop with #{token.kind}"

      case token.kind
      when :section_start
        section = try_section(tokens)
        if section.level > 0 and elements.last.instance_of? Section
          elements.last.append section
        else
          elements << section
        end
      when :newline
        tokens.pop
      when :attribute_start
        attributes << parse_attribute(tokens)
      when :block_start
        elements << parse_block(tokens)
      else
        elements << parse_paragraph(tokens)
      end
    end
    elements
  end

  def OrgParsing.try_section tokens
    #puts "Start section"
    tokens.pop
    level = 0
    token = tokens.pop
    while token.is? :asterisk
      level += 1
      token = tokens.pop
    end
    title = tokens_to_s [token] + tokens.pop_until { |t| t.is? :newline }
    tokens.pop

    properties = {}
    if tokens.peek.is? :colon
      properties = parse_section_properties tokens
    end
    Section.new level, title, parse(tokens, :section_start), properties
  end

  def OrgParsing.parse_attribute tokens
    tokens.pop
    t = tokens.pop_expected :word
    tokens.pop_expected :colon
    tokens.pop_while { |tok| tok.is? :whitespace }

    case t.value
    when "HTML"
      result = parse_html_attribute tokens
    else
      raise OrgParseError, "#{t.loc}: Unknown attribute type #{t.value}"
    end

    tokens.pop_expected :newline
    result
  end

  def OrgParsing.parse_html_attribute tokens
    tokens.pop_expected :colon
    t = tokens.pop_expected :word

    raise OrgParseError, "#{t.loc}: Can only handle style in html attribute right now" unless t.value == "style"


    HTMLStyleAttribute.new tokens_to_s(tokens.pop_until { |tok| tok.is? :newline })
  end

  def OrgParsing.parse_section_properties tokens
    properties = {}
    tokens.pop_expected :colon
    tmp = tokens.pop_expected :word
    raise OrgParseError, "#{tmp.loc}: expected 'PROPERTIES' got #{tmp.value}" unless tmp.value == "PROPERTIES"
    tokens.pop_expected :colon
    tokens.pop_expected :newline

    while true
      tokens.pop_expected :colon
      key = tokens.pop_until { |t| t.is? :colon }
      tokens.pop_expected :colon
      break if key[0].value == "END"

      tokens.pop_while { |t| t.is? :whitespace }
      value = tokens.pop_until { |t| t.is? :newline }
      tokens.pop_expected :newline

      properties[tokens_to_s key] = tokens_to_s value
    end

    properties
  end

  def OrgParsing.parse_text tokens
    tokens_to_s tokens.pop_while { |t| t.is_text? }
  end

  def OrgParsing.parse_link_target tokens
    tokens_to_s tokens.pop_while { |t| t.is_text? or t.is? :slash }
  end

  def OrgParsing.parse_special_text tokens
    delim = tokens.pop.kind
    text = parse_text tokens
    t = tokens.pop

    raise OrgParseError, "Expected SpecialText delimiter #{delim} but found #{t.value}" unless t.is? delim

    case delim
    when :asterisk
      SpecialText.new :bold, text
    when :slash
      SpecialText.new :italic, text
    end
  end

  def OrgParsing.parse_paragraph tokens
    elements = []
    # remove this until loop. Instead, loop only in the outermost function and write some logic there to merge all parsed elements
    until tokens.no_tokens? or tokens.peek.is_paragraph_end?
      t = tokens.peek
      case
      when t.is_text_element?
        elements += parse_text_elements(tokens)
      else
        raise OrgParseError, "#{t.loc}: Unknown token '#{t}'"
      end
    end
    Paragraph.new elements
  end

  def OrgParsing.parse_link tokens
    tokens.pop_expected :left_square_brace
    tokens.pop_expected :left_square_brace

    target = parse_link_target tokens

    tokens.pop_expected :right_square_brace
    tokens.pop_expected :left_square_brace

    text = parse_text tokens

    tokens.pop_expected :right_square_brace
    tokens.pop_expected :right_square_brace

    Link.new target, text
  end

  def OrgParsing.parse_text_elements tokens
    elements = []
    while tokens.has_tokens? and tokens.peek.is_text_element?
      t = tokens.peek
      case
      when t.is_special_text_delimiter?
        elements << parse_special_text(tokens)
      when t.is_text?
        elements << parse_text(tokens)
      when t.is?(:left_square_brace)
        elements << parse_link(tokens)
      end
      tokens.pop_if { |x| x.is? :newline }
    end
    elements
  end

  def OrgParsing.parse_block tokens
    tokens.pop_expected :block_start
    t = tokens.pop_expected :word
    tokens.pop_expected :newline

    expected = t.value
    case expected
    when "COMMENT"
      result = Comment.new parse_text_elements(tokens)
    when "QUOTE"
      result = parse_quote_block tokens
    else
      raise OrgParseError, "#{t.loc}: Unknown block type #{t.value}"
    end

    tokens.pop_expected :block_end
    t = tokens.pop_expected :word
    raise OrgParseError, "#{t.loc}: Expected '#{expected}' to end block. Found #{t.value} instead." unless t.value == expected
    tokens.pop_expected :newline
    result
  end

  def OrgParsing.parse_quote_block tokens
    text = parse_text_elements tokens
    quotee = nil
    if tokens.pop_if { |t| t.is? :quotee_start }
      tokens.pop_while { |t| t.is? :whitespace }
      quotee = parse_text_elements tokens
    end
    Quote.new text, quotee
  end

  def OrgParsing.s_to_date s
    #raise InvalidTokenError unless token.is? :date
    /<(?<y>\d{4})-(?<m>\d{2})-(?<d>\d{2})>/ =~ s
    Date.new y, m, d
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
    "<a href=\"#{@target}\" target=\"_blank\">#{@text}</a>"
  end
end

class HTMLStyleAttribute
  attr_reader :style
  def initialize str
    @style = str
  end
end
