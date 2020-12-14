require_relative 'types'
require_relative 'tokenizer'
require_relative 'token_helpers'

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
    tokens_to_s tokens.pop_until { |t| t.is? :right_square_brace }
  end

  def OrgParsing.parse_special_text tokens
    delim = tokens.pop
    text = parse_text tokens
    t = tokens.pop

    unless t.is? delim.kind
      # parsing special text failed
      return delim.value + text
    end

    case delim.kind
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

    unless tokens.peek.is? :left_square_brace
      # this isn't a link; parse text instead.
      return "[" + parse_text(tokens)
    end
    tokens.pop

    target = parse_link_target tokens
    tokens.pop_expected :right_square_brace

    text = nil
    if tokens.pop_if { |t| t.is? :left_square_brace }
      text = tokens_to_s(tokens.pop_until { |t| t.is? :right_square_brace })
      tokens.pop
    end

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
