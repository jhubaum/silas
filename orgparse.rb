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
    while tokens.pop_if { |t| t.is? :attribute_start } != nil
      val = tokens.pop_expected :word
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

module OrgParsing
  def OrgParsing.parse tokens, until_token=nil
    #puts "Enter Parse (until token: #{until_token})"
    elements = []
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
      else
        elements << try_paragraph(tokens)
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
    raise OrgParseError, "expected whitespace" unless token.is? :whitespace
    title = tokens_to_s tokens.pop_until { |t| t.is? :newline }
    tokens.pop
    Section.new level, title, parse(tokens, :section_start)
  end

  def OrgParsing.parse_special_text tokens
  end

  def OrgParsing.parse_text tokens
    tokens_to_s tokens.pop_while { |t| t.is_text? }
  end

  def OrgParsing.parse_special_text tokens
    delim = tokens.pop.kind
    text = parse_text tokens
    t = tokens.pop

    raise OrgParseError, "Expected SpecialText delimiter #{expected} but found #{t.value}" unless t.is? delim

    case delim
    when :asterisk
      SpecialText.new :bold, text
    when :slash
      SpecialText.new :italic, text
    end
  end

  def OrgParsing.try_paragraph tokens
    elements = []
    until tokens.no_tokens? or tokens.peek.is_any? [:newline, :section_start]
      case
      when tokens.peek.is_text?
        elements << parse_text(tokens)
      when tokens.peek.is_special_text_delimiter?
        elements << parse_special_text(tokens)
      else
        raise OrgParseError, "Unkown token '#{tokens.peek.value}' to start paragraph (#{tokens.peek.loc})"
      end
      tokens.pop_if { |t| t.is? :newline }
    end
    Paragraph.new elements
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
  attr_reader :level, :title, :children

  def initialize level, title, children
    @level = level
    @title = title
    @children = children
  end

  def append element
    @children << element
  end

  def heading
    "<h#{@level+2}>#{@title}</h#{@level+2}>"
  end

  def iterate_elements &block
    @children.each do |elem|
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
