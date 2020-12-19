require_relative 'types'
require_relative 'tokenizer'
require_relative 'token_helpers'

class OrgParser
  attr_reader :preamble, :elements

  def OrgParser.parse_file filename
    raise OrgReadFileError, "file '#{filename}' does not exist" unless File.file? filename

    parser = OrgParser.new File.open(filename).read
    OrgFile.send :new, parser.preamble, parser.elements
  end

  def OrgParser.parse_expression expression
    parser = OrgParser.new expression

    parser.preamble ?
      [parser.preamble] + parser.elements :
      parser.elements
  end

  def initialize expression
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
    else
      elems = []
      until @tokens.no_tokens? or @tokens.peek.is_paragraph_end?
        elems += parse_text_line
      end
      Paragraph.new elems
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

    Section.new level, title, elements, properties
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
                                       :word, :newline])[1].value

    case block_type
    when "COMMENT"
      res = parse_comment
    when "QUOTE"
      res = parse_quote
    else
      raise OrgParseError, "Unknown block type #{block_type}"
    end

    @tokens.pop_if { |t| t.is? :newline }

    end_type = @tokens.pop_expected([:block_end, :word])[1].value
    raise OrgParseError, "#{t.loc}: Expected '#{block_type}' to end block." unless block_type == end_type
    res
  end

  def parse_comment
    elements = []
    until @tokens.peek.is? :block_end
      elements += parse_text_line
    end
    Comment.new elements
  end

  def parse_quote
    elements = []
    quotee = nil
    until @tokens.peek.is_any? [:block_end, :quotee_start]
      elements +=  parse_text_line
    end

    if @tokens.peek.is? :quotee_start
      @tokens.pop
      quotee = parse_text_line
    end

    Quote.new elements, quotee
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

    l = Link.new target, text
    l.attributes = pop_attributes if @attributes.length > 0
    l
  end

  def parse_special_text type
    text = parse_plain_text_until_and_pop @tokens.pop.kind
    SpecialText.new type, text
  end

  def parse_plain_text_until_and_pop kind
    tokens = @tokens.pop_until { |t| t.is? kind or t.is_line_end? }

    tok = @tokens.peek
    raise OrgParseError, "#{tok.loc}: Expected #{kind} but found line end" unless tok.is? kind

    @tokens.pop
    tokens_to_s tokens
  end
end
