require_relative 'types'
require_relative 'tokenizer'
require_relative 'token_helpers'

class OrgParser
  attr_reader :preamble, :elements

  def OrgParser.parse_file file, filename
    raise OrgReadFileError, "file '#{filename}' does not exist" unless File.file? filename

    puts "Start parsing file '#{filename}'"

    parser = OrgParser.new file, File.open(filename).read
    return parser.preamble, parser.elements
  end

  def OrgParser.parse_expression expression
    parser = OrgParser.new expression

    parser.preamble ?
      [parser.preamble] + parser.elements :
      parser.elements
  end

  def initialize file, expression
    @file = file
    @tokens = Tokenizer.tokenize expression
    @preamble = parse_preamble
    @elements = []
    @attributes = []

    while @tokens.has_tokens?
      elem = parse_text_block
      @elements << elem unless elem == nil
    end
  end

  private
  def parse_preamble
    if @tokens.has_tokens? and @tokens.peek.is? :quotee_start
      raise "This file needs to be adjusted"
    end
    preamble = { }
    while @tokens.pop_if { |t| t.is? :preamble_start }
      val = @tokens.pop_expected(:word).value
      @tokens.pop_expected :colon
      @tokens.pop_if { |t| t.is? :whitespace }
      elems = @tokens.pop_until { |t| t.is? :newline }
      @tokens.pop

      preamble[val.to_sym] = tokens_to_s elems
    end

    # convert dates
    [:published, :lastedit].each do |k|
      preamble[k] = Date.from_s preamble[k] if preamble.key? k
    end

    preamble
  end

  def pop_attributes
    res = @attributes
    @attributes = []
    res
  end

  def parse_text_block
    @tokens.pop_while { |t| t.is? :newline }
    return nil if @tokens.no_tokens?

    case @tokens.peek.kind
    when :section_start
      parse_section
    when :attribute_start
      @attributes << parse_attribute
      nil
    when :block_start
      parse_block
    when :plus, :minus, :number
      parse_list
    else
      elems = []
      until @tokens.no_tokens? or @tokens.peek.is_paragraph_end?
        elems += parse_text_line
      end
      Paragraph.new @file, elems
    end
  end

  def parse_section
    level = section_level
    @tokens.pop
    title = parse_text_line.map(&:to_s).join("")
    @tokens.pop_if { |t| t.is? :newline }
    properties = parse_section_properties

    elements = []
    until @tokens.no_tokens? or
          (@tokens.peek.is? :section_start and section_level <= level)
      elem = parse_text_block
      elements << elem unless elem == nil
    end

    Section.new @file, level: level,
    title: title, elements: elements, properties: properties
  end

  def section_level
    t = @tokens.peek
    raise OrgParseError, "Called section_level on #{t.kind}" unless t.is? :section_start

    t.value.length - 3
  end

  def parse_section_properties
    properties = {}
    unless @tokens.peek.is? :colon
      return properties
    end

    @tokens.pop
    toc = @tokens.pop_expected :word
    raise OrgParseError, "#{toc.loc}: expected 'PROPERTIES' got #{toc.value}" unless toc.value == "PROPERTIES"
    @tokens.pop_expected [:colon, :newline]

    while true
      @tokens.pop_expected :colon
      key = @tokens.pop_until { |t| t.is? :colon }
      @tokens.pop
      break if key[0].value == "END"

      @tokens.pop_while { |t| t.is? :whitespace }
      value = @tokens.pop_until { |t| t.is? :newline }
      @tokens.pop

      properties[tokens_to_s key] = tokens_to_s value
    end

    properties
  end

  def parse_attribute
    attr_start = @tokens.pop_expected([:attribute_start, :word,
                                       :colon, :whitespace])
    case attr_start[1].value
    when "HTML"
      parse_html_attribute
    else
      raise OrgParseError, "#{t.loc}: Unknown attribute type #{t.value}"
    end
  end

  def parse_html_attribute
    t = @tokens.pop_expected([:colon, :word])[1]
    raise OrgParseError, "#{t.loc}: Can only handle style in html attribute right now" unless t.value == "style"

    style = tokens_to_s @tokens.pop_until { |tok| tok.is? :newline }
    HTMLStyleAttribute.new style
  end

  def parse_block
    block_type = @tokens.pop_expected([:block_start,
                                       :word])[1].value

    args = []
    while @tokens.peek.is? :whitespace
      @tokens.pop
      args << @tokens.pop_expected(:word).value
    end
    @tokens.pop_expected :newline

    case block_type
    when "COMMENT"
      res = parse_comment args
    when "QUOTE"
      res = parse_quote args
    when "SRC"
      res = parse_src args
    else
      raise OrgParseError, "Unknown block type #{block_type}"
    end

    @tokens.pop_if { |t| t.is? :newline }

    end_type = @tokens.pop_expected([:block_end, :word])[1].value
    raise OrgParseError, "#{t.loc}: Expected '#{block_type}' to end block." unless block_type == end_type
    res
  end

  def parse_comment args
    elements = []
    until @tokens.peek.is? :block_end
      elements += parse_text_line
    end
    Comment.new @file, elements
  end

  def parse_quote args
    elements = []
    quotee = nil
    until @tokens.peek.is_any? [:block_end, :quotee_start]
      elements +=  parse_text_line
    end

    if @tokens.peek.is? :quotee_start
      @tokens.pop
      @tokens.pop_while { |t| t.is? :whitespace }
      quotee = parse_text_line
    end

    Quote.new @file, elements, quotee
  end

  def parse_src args
    lang = args
    code = @tokens.pop_until { |t| t.is? :block_end }.map(&:value).join("")

    CodeBlock.new lang, code
  end

  def parse_list indentation=0
    type = parse_list_type
    entries = []
    loop do
      entries << parse_list_entry(indentation + type.indentation)
      break if @tokens.no_tokens? or list_is_finished type, entries.length
    end
    List.new @file, type, entries
  end

  def parse_list_type
    tok = @tokens.pop
    case tok.kind
    when :minus, :plus
      @tokens.pop_expected :whitespace
      ListType::Unordered.new tok.kind
    when :number
      raise "#{tok.loc}: ordered list didn't start with `1`. Probably a parsing error" unless tok.value.to_i == 1
      @tokens.pop_expected [:dot, :whitespace]
      ListType::Ordered.new
    else
      raise "#{tok.loc}: Unknown char for list start"
    end
  end

  def parse_list_entry indentation
    elements = [parse_text_line]
    while parse_indentation(indentation) and @tokens.has_tokens?
      if @tokens.peek.is_list_start?
        elements << parse_list(indentation)
      else
        elements << parse_text_line
      end
    end
    elements
  end

  def parse_indentation indentation
    @tokens.start_checkpoint
    begin
      @tokens.pop_expected [:whitespace]*indentation
    rescue TokenListError
      @tokens.revert_checkpoint
      false
    else
      true
    end
  end

  def list_is_finished type, count
    if type.is_next_entry @tokens.peek, count
      @tokens.pop
      @tokens.pop_while { |t| t.is_any? [:whitespace, :dot] }
      return false
    else
      return true
    end
  end

  def parse_text_line
    elements = []
    until @tokens.peek.is_line_end?
      @tokens.start_checkpoint
      begin
        new = parse_text_element
      rescue OrgParseError, TokenListError => error
        puts "Warning: #{error}"
        new = @tokens.use_checkpoint_as_s
      ensure
        if elements.last.instance_of? String and
          new.instance_of? String
            elements[-1] += new
        else
          elements << new
        end
      end
    end
    @tokens.pop_if { |t| t.is? :newline }
    elements
  end

  def parse_text_element
    case @tokens.peek.kind
    when :left_square_brace
      parse_link
    when :asterisk
      parse_special_text :bold
    when :slash
      parse_special_text :italic
    else
      @tokens.pop.value
    end
  end

  def parse_link
    @tokens.pop
    @tokens.pop_expected :left_square_brace

    target = parse_plain_text_until_and_pop :right_square_brace

    text = nil
    if @tokens.pop_if { |t| t.is? :left_square_brace }
      text = parse_plain_text_until_and_pop :right_square_brace
    end
    @tokens.pop_expected :right_square_brace

    l = Link.new @file, target, text
    l.attributes = pop_attributes if @attributes.length > 0
    l
  end

  def parse_special_text type
    text = parse_plain_text_until_and_pop @tokens.pop.kind
    SpecialText.new @file, type, text
  end

  def parse_plain_text_until_and_pop kind
    tokens = @tokens.pop_until { |t| t.is? kind or t.is_line_end? }

    tok = @tokens.peek
    raise OrgParseError, "#{tok.loc}: Expected #{kind} but found line end" unless tok.is? kind

    @tokens.pop
    tokens_to_s tokens
  end
end
